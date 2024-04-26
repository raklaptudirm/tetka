// Copyright © 2024 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};

use std::default::Default;

use crate::{error, inbuilt, Parameter};

use super::{Command, FlagValues, RunError, RunErrorType};

/// Client represents an UAI engine client. It can accept and parse commands
/// from the GUI and send commands to the GUI though its input and output.
/// Commands sent from the GUI are automatically parsed and executed according
/// to the Command schema provided by the user to the Client.
pub struct Client<T: Send, E: RunError> {
    initial_context: inbuilt::Context,
    commands: HashMap<String, Command<T, E>>,
}

impl<T: Send + 'static, E: RunError + 'static> Client<T, E> {
    /// start starts the Client so that it can now accept Commands from the GUI and
    /// send Commands back to the GUI as necessary. The Client will return only if
    /// it encounters a fatal error while running a command ([`RunErrorType::Fatal`])
    /// or one of the commands asks the Client to quit ([`RunErrorType::Quit`]).
    pub fn start(&self, context: T) {
        // The GUI sends commands to the stdin.
        let stdin = io::stdin();

        // Make the context thread safe to allow commands to run in parallel.
        let context = Arc::new(Mutex::new(context));
        let our_ctx = Arc::new(Mutex::new(self.initial_context.clone()));

        // Iterate over the lines in the input, since Commands for the GUI are
        // separated by newlines and we want to parse each Command separately.
        'reading: for line in stdin.lock().lines() {
            // Split the Command into parts by whitespace.
            let line = line.unwrap();
            let parts = line.split_whitespace().collect::<Vec<&str>>();

            let (cmd_name, args) = (parts[0], &parts[1..]);

            // Try to find a Command with the given name.
            let cmd = self.commands.get(cmd_name);
            if cmd.is_some() {
                // Parsing complete, run the Command and handle any errors.
                match self.run(cmd.unwrap(), &context, args) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                        if err.should_quit() {
                            break 'reading;
                        }
                    }
                };

                continue 'reading;
            }

            // Try to find a Command with the given name.
            let cmd = inbuilt::COMMANDS.get(cmd_name);
            if cmd.is_some() {
                // Parsing complete, run the Command and handle any errors.
                match self.run(cmd.unwrap(), &our_ctx, args) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                        if err.should_quit() {
                            break 'reading;
                        }
                    }
                };

                continue 'reading;
            }

            // Command not found, return error and continue.
            println!("info error command {} not found", cmd_name);
            continue 'reading;
        }
    }

    pub fn run<CT: Send + 'static, CE: RunError + 'static>(
        &self,
        cmd: &Command<CT, CE>,
        context: &Arc<Mutex<CT>>,
        args: &[&str],
    ) -> Result<(), RunErrorType> {
        // Initialize an empty list of the Command's Flags' values.
        let mut flags: FlagValues = Default::default();

        let mut args = args;

        // The arguments have the following format:
        // { flag_name { flag_arg... } ... }
        while !args.is_empty() {
            let flag_name = args[0]; // The first arg has to be a flag name.
            args = &args[1..]; // Remove the flag name from the rest of the args.

            // Try to find a flag with the given name.
            let flag = cmd.flags.get(flag_name);
            if flag.is_none() {
                // Flag not found, return error and continue.
                return error!("info error flag {} not found", flag_name);
            }

            // The Option<Flag> in not None, so it can be safely unwrapped.
            let flag = flag.unwrap();

            // Find the number of arguments the Flag expects.
            let yank = flag.collect(args);

            // Check if args has the required number of arguments.
            if args.len() < yank {
                return error!(
                    "info error flag {} expects {} arguments, found {}",
                    flag_name,
                    yank,
                    args.len(),
                );
            }

            // Collect that number of arguments from the remaining args.
            let collected = &args[..yank];
            flags.insert(flag_name, *flag, collected);
            args = &args[yank..];
        }

        // Parsing complete, run the Command and handle any errors.
        cmd.run(context, flags, self.initial_context.option_values.clone())
            .map_err(|e| e.into())
    }
}

impl<T: Send, E: RunError> Client<T, E> {
    /// new creates a new [Client]. The Client can be configured using builder
    /// methods like [`Client::command`].
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .command("go", go_cmd)
    ///     .command("perft", perft_cmd);
    /// ```
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    pub fn new() -> Self {
        Client::<T, E> {
            initial_context: Default::default(),
            commands: HashMap::new(),
        }
    }

    /// command adds the given Command to the given Client. After this, the Client
    /// will be able to parse and run that Command when such a request is sent from
    /// the GUI.
    ///
    /// command takes ownership of the given Client value and returns that ownership
    /// after adding the given Command. This allows the usage of the builder pattern
    /// while creating a new [Client].
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .command("go", go_cmd)
    ///     .command("perft", perft_cmd);
    /// ```
    pub fn command(mut self, name: &str, cmd: Command<T, E>) -> Self {
        self.commands.insert(name.to_string(), cmd);
        self
    }

    pub fn option(mut self, name: &str, option: Parameter) -> Self {
        self.initial_context
            .options
            .insert(name.to_string(), option.clone());
        self.initial_context
            .option_values
            .insert_default(name.to_string(), &option);
        self
    }

    pub fn engine(mut self, name: &str) -> Self {
        self.initial_context.engine = name.to_owned();
        self
    }

    pub fn author(mut self, name: &str) -> Self {
        self.initial_context.author = name.to_owned();
        self
    }
}
