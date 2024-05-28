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
use std::default::Default;
use std::io::{self, BufRead};

use crate::context::Context;
use crate::inbuilt::new_guarded_ctx;
use crate::{error, flag, inbuilt, Command, GuardedBundledCtx, Parameter, RunError};

/// Client represents an UXI engine client. It can accept and parse commands
/// from the GUI and send commands to the GUI though its input and output.
/// Commands sent from the GUI are automatically parsed and executed according
/// to the Command schema provided by the user to the Client. The client supports
/// any UXI type protocol, including but not limited to UCI, UGI, and UAI.
pub struct Client<T: Send> {
    initial_context: Context,
    commands: HashMap<String, Command<T>>,
}

impl<T: Send + 'static> Client<T> {
    /// start starts the Client so that it can now accept Commands from the GUI and
    /// send Commands back to the GUI as necessary. The Client will return only if
    /// it encounters a fatal error while running a command ([`RunError::Fatal`])
    /// or one of the commands asks the Client to quit ([`RunError::Quit`]).
    pub fn start(&self, context: T) {
        // The GUI sends commands to the stdin.
        let stdin = io::stdin();

        // Make the context thread safe to allow commands to run in parallel.
        let context = new_guarded_ctx(context, self.initial_context.clone());

        // Iterate over the lines in the input, since Commands for the GUI are
        // separated by newlines and we want to parse each Command separately.
        'reading: for line in stdin.lock().lines() {
            // Split the Command into parts by whitespace.
            let line = line.unwrap();
            let parts = line.split_whitespace().collect::<Vec<&str>>();

            let (cmd_name, args) = (parts[0], &parts[1..]);

            // Try to find a Command with the given name.
            let cmd = match self.commands.get(cmd_name) {
                Some(c) => c,
                None => {
                    // Command not found, return error and continue.
                    println!("info error command {} not found", cmd_name);
                    continue 'reading;
                }
            };

            // Parsing complete, run the Command and handle any errors.
            if let Err(err) = self.run(cmd, &context, args) {
                println!("{}", err);
                if err.should_quit() {
                    break 'reading;
                }
            };
        }
    }

    /// run runs the given Command with the given [context](GuardedBundledCtx) and
    /// the given arguments. This function is used internally when a Client is
    /// started. Only use this function if you know what you are doing.
    fn run(
        &self,
        cmd: &Command<T>,
        context: &GuardedBundledCtx<T>,
        args: &[&str],
    ) -> Result<(), RunError> {
        // Initialize an empty list of the Command's Flags' values.
        let mut flags: flag::Values = Default::default();

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
        cmd.run(context, flags)
    }
}

impl<T: Send> Client<T> {
    /// new creates a new [Client]. The Client can be configured using builder
    /// methods like [`Client::command`], [`Client::option`], etc. These functions
    /// take the ownership of the given Client value and return that ownership
    /// after adding the given option. This allows the usage of the builder pattern
    /// while creating a new [Client].
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .command("go", go_cmd)
    ///     .command("perft", perft_cmd);
    /// ```
    #[allow(clippy::new_without_default)]
    #[rustfmt::skip]
    pub fn new() -> Self {
        Client::<T> {
            initial_context: Default::default(),
            commands: HashMap::from([
                ("quit".to_owned(), inbuilt::commands::quit()),
                ("isready".to_owned(), inbuilt::commands::isready()),
                ("ugi".to_owned(), inbuilt::commands::ugi()),
                ("setoption".to_owned(), inbuilt::commands::setoption()),
                ("options".to_owned(), inbuilt::commands::options()),
            ]),
        }
    }

    /// command adds the given Command to the given Client. After this, the Client
    /// will be able to parse and run that Command when such a request is sent from
    /// the GUI.
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .command("go", go_cmd)
    ///     .command("perft", perft_cmd);
    /// ```
    pub fn command(mut self, name: &str, cmd: Command<T>) -> Self {
        self.commands.insert(name.to_string(), cmd);
        self
    }

    /// option adds the given [option](Parameter) to the given Client. This will
    /// register that option with the Client, and those options can then be set by
    /// GUIs using the engine with a UXI type protocol. The options will also be
    /// made available in the [context](crate::BundledCtx) so that Commands can
    /// retrieve and use their values.
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .option("Hash", Parameter::Spin(16, 1, 33554432))
    ///     .option("Threads", Parameter::Spin(1, 1, 1024));
    /// ```
    pub fn option(mut self, name: &str, option: Parameter) -> Self {
        self.initial_context
            .options
            .insert(name.to_string(), option.clone());
        self.initial_context
            .option_values
            .insert_default(name.to_string(), &option);
        self
    }

    /// protocol configures the Client to support the given UXI protocol. UXI
    /// protocols have a name in the format "uxi", where the x can be replaced by
    /// any lowercase alphabet, usually the leading letter of the name of the game
    /// played by the engine.
    ///
    /// Some popular protocols are "uci" for Chess and "uai" for Ataxx. The name
    /// "ugi" however is reserved and may not be used as a protocol name. This is
    /// because ugi is the identifier for the Universal Game Interface, which is a
    /// game-agnostic protocol which is supported by all Clients by default.
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .protocol("uai");
    /// ```
    pub fn protocol(mut self, name: &str) -> Self {
        assert!(!self.commands.contains_key(name));

        // Move the previous protocol identifier command to the new name.
        self.commands.remove(&self.initial_context.protocol);
        self.commands
            .insert(name.to_string(), inbuilt::commands::uxi());

        // Change the protocol name.
        name.clone_into(&mut self.initial_context.protocol);
        self
    }

    /// engine sets the name of the engine for the Client.
    ///
    /// This information is reported to the GUI when the uxi Command is invoked, and
    /// is also accessible by Commands through their [bundles](crate::BundledCtx).
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .protocol("uai");
    /// ```
    pub fn engine(mut self, name: &str) -> Self {
        name.clone_into(&mut self.initial_context.engine);
        self
    }

    /// author sets the name of the author of the Client's engine.
    ///
    /// This information is reported to the GUI when the uxi Command is invoked, and
    /// is also accessible by Commands through their [bundles](crate::BundledCtx).
    /// ```rust,ignore
    /// let client = Client::new()
    ///     .protocol("uai");
    /// ```
    pub fn author(mut self, name: &str) -> Self {
        name.clone_into(&mut self.initial_context.author);
        self
    }
}
