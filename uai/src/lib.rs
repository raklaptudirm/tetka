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

use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;

/// Client represents an UAI engine client. It can accept and parse commands
/// from the GUI and send commands to the GUI though its input and output.
/// Commands sent from the GUI are automatically parsed and executed according
/// to the Command schema provided by the user to the Client.
pub struct Client<T: Send, E: RunError> {
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

        // Iterate over the lines in the input, since Commands for the GUI are
        // separated by newlines and we want to parse each Command separately.
        'reading: for line in stdin.lock().lines() {
            // Get the full String version of the Command.
            let line = line.unwrap();

            // Split the Command into parts by whitespace.
            let parts = line.split_whitespace().collect::<Vec<&str>>();

            let cmd_name = parts[0]; // The first part is the Command name.
            let mut args = &parts[1..]; // The others are flags and their args.

            // Try to find a Command with the given name.
            let cmd = self.commands.get(cmd_name);
            if cmd.is_none() {
                // Command not found, return error and continue.
                println!("info error command {} not found", cmd_name);
                continue 'reading;
            }

            // The Option<Command> is not None, so it can be safely unwrapped.
            let cmd = cmd.unwrap();

            // Initialize an empty list of the Command's Flags' values.
            let mut flags: FlagValues = Default::default();

            // The arguments have the following format:
            // { flag_name { flag_arg... } ... }
            while !args.is_empty() {
                let flag_name = args[0]; // The first arg has to be a flag name.
                args = &args[1..]; // Remove the flag name from the rest of the args.

                // Try to find a flag with the given name.
                let flag = cmd.flags.get(flag_name);
                if flag.is_none() {
                    // Flag not found, return error and continue.
                    println!("info error flag {} not found", flag_name);
                    continue 'reading;
                }

                // The Option<Flag> in not None, so it can be safely unwrapped.
                let flag = flag.unwrap();

                // Find the number of arguments the Flag expects.
                let yank = flag.collect(args);

                // Check if args has the required number of arguments.
                if args.len() < yank {
                    println!(
                        "info error flag {} expects {} arguments, found {}",
                        flag_name,
                        yank,
                        args.len(),
                    );
                    continue 'reading;
                }

                // Collect that number of arguments from the remaining args.
                let collected = &args[..yank];
                flags.insert(flag_name, *flag, collected);
                args = &args[yank..];
            }

            // Parsing complete, run the Command and handle any errors.
            match cmd.run(&context, flags) {
                Ok(_) => (),
                Err(err) => match err.into() {
                    // Quit is a directive to quit the Client, so break
                    // out of the main Command loop reading from stdin.
                    RunErrorType::Quit => break,

                    // Command encountered some simple error, report it
                    // to the GUI and continue parsing Commands.
                    RunErrorType::Error(o_o) => println!("info error {}", o_o),

                    // Fatal error encountered, report it to the GUI and quit
                    // the Client, since this error can't be recovered from.
                    RunErrorType::Fatal(o_o) => {
                        println!("info error {}", o_o);
                        break;
                    }
                },
            };
        }
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
    pub fn new() -> Self {
        Client::<T, E> {
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
}

/// RunFn<T, E> represents the run function of a Command. This function is called
/// with the context (`Arc<Mutex<T>>`) and the flag values ([`FlagValues`]) whenever
/// the Command is to be executed. It returns a `Result<(), E>` where `E` implements
/// [`RunError`] and is the error type for the [Client].
pub type RunFn<T, E> = fn(Arc<Mutex<T>>, FlagValues) -> Result<(), E>;

/// Command represents a runnable UAI command. It contains all the metadata
/// needed to parse and verify a Command request from the GUI for a Command, and
/// to run that Command with the current context and the provided flag values.
/// `T` is the context type of the [Client], while `E` is the error type. `E`
/// must implement the [`RunError`] trait to be usable.
pub struct Command<T, E: RunError> {
    /// run_fn is the function used to run this Command.
    pub run_fn: RunFn<T, E>,
    /// flags is the schema of the Flags this Command accepts.
    pub flags: HashMap<String, Flag>,
    /// parallel says whether to run this command in a separate thread.
    pub parallel: bool,
}

impl<T: Send + 'static, E: RunError + 'static> Command<T, E> {
    /// run runs the current Command with the given context and flag values.
    /// A new thread is spawned and detached to run parallel Commands. It returns
    /// the error returned by the Command's execution, or [`Ok`] for parallel.
    pub fn run(&self, context: &Arc<Mutex<T>>, flags: FlagValues) -> Result<(), E> {
        // Clone values which might be moved by spawning a new thread.
        let context = Arc::clone(context);
        let func = self.run_fn;

        if self.parallel {
            // If the Command is supposed to be run in parallel, spawn a new
            // thread and detach it for its execution. Syncing with the thread
            // should be handled by the user using the context.
            thread::spawn(move || func(context, flags));
            return Ok(());
        }

        // Run the synchronous Command and return its error.
        func(context, flags)
    }
}

impl<T: Send, E: RunError> Command<T, E> {
    /// new creates a new Command with the given run function.
    ///
    /// By default the flag schema is empty the the Command is run synchronously.
    ///
    /// Further configuration of the Command can be done using the builder style
    /// methods provided on Command, like [`Self::flag`] and [`Self::parallelize`].
    /// These methods take the ownership of the given Command, make the necessary
    /// changes and then return it. These allows them to be chained in builder
    /// pattern style to create fully configured Commands.
    /// ```rust,ignore
    /// let cmd: Command<T, E> =
    ///     // new invocation to create a Command. In this example, a very
    ///     // simple run function which returns `Ok(())` is provided.
    ///     Command::new(|_ctx, _flg| Ok(()))
    ///         // Add flags to the Command's flag schema.
    ///         .flag("flag1", Flag::Boolean)
    ///         .flag("flag2", Flag::Singular)
    ///         .flag("flag3", Flag::Array(10))
    ///         .flag("flag4", Flag::Variadic)
    ///         // Make the command run in parallel.
    ///         .parallelize(true);
    /// ```
    pub fn new(func: RunFn<T, E>) -> Command<T, E> {
        Command {
            run_fn: func,
            flags: Default::default(),
            parallel: false,
        }
    }

    /// flag adds the given Flag to the Command's Flag schema.
    ///
    /// After a flag is added to a Command, the Client will automatically parse that
    /// flag and verify its arguments whenever that Command is invoked.
    ///
    /// flag is a builder method. Refer to the documentation of [`Command::new`] for
    /// more information about [Command]'s builder methods.
    pub fn flag(mut self, name: &str, flag: Flag) -> Self {
        self.flags.insert(name.to_string(), flag);
        self
    }

    /// parallelize sets if the Command to be run in a separate thread.
    ///
    /// Synchronization of a parallel Command should be done with the help of the
    /// mutex-locked context that is provided to the Command's run function. The
    /// Client continues in the main thread while a parallel Command is running.
    ///
    /// parallelize is a builder method. Refer to the documentation of
    /// [`Command::new`] for more information about [Command]'s builder methods.
    pub fn parallelize(mut self, y: bool) -> Self {
        self.parallel = y;
        self
    }
}

/// RunError is the interface which the Client uses to understand custom errors
/// returned by a Command. It allows the user to implement their own error types
/// while allowing the Client to interpret and use those errors. This is
/// achieved by requiring conversions from and into [`RunErrorType`].
pub trait RunError: Send + From<RunErrorType> + Into<RunErrorType> {}

/// `quit!()` resolves to a [`Err(~RunErrorType::Quit)`](RunErrorType::Quit)
/// kind of error, and thus can be called by itself inside a Command to instruct
/// the Client to quit itself and stop executing commands.
#[macro_export]
macro_rules! quit {
    () => {
        Err(ataxx_uai::RunErrorType::Quit.into())
    };
}

/// `error!()` resolves to a [`Err(~RunErrorType::Error)`](RunErrorType::Error)
/// kind of error, and thus can be called by itself inside a Command to exit the
/// Command with the given error. It supports the same arguments as the
/// [`format!`] macro.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
            Err(ataxx_uai::RunErrorType::Error(format!($($arg)*)).into())
        }
    };
}

/// `error_val!()` resolves to a `E` value which represents a
/// [`RunErrorType::Error`] kind of error. It can be used to simplify creating
/// such error values where necessary, for example in conversion functions. If
/// this value will be returned from a Command, use the [`error!`] macro instead.
/// This macro has the same arguments as the [`error!`] macro.
#[macro_export]
macro_rules! error_val {
    ($($arg:tt)*) => {
        {
            ataxx_uai::RunErrorType::Error(format!($($arg)*)).into()
        }
    };
}

/// `fatal!()` resolves to a [`Err(~RunErrorType::Fatal)`](RunErrorType::Fatal)
/// kind of error, and thus can be called by itself inside a Command to exit the
/// Command with the given error and to quit the Client. It is similar to the
/// [`error!`] macro and supports the same arguments.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {
        Err(ataxx_uai::RunErrorType::Fatal(format!($($arg)*)).into())
    };
}

/// RunErrorType is the error that is used internally in Client. All user errors
/// must support conversion into this type so that the Client can handle them.
#[derive(Clone)]
pub enum RunErrorType {
    /// Quit directs the Client to quit itself, without reporting any errors.
    Quit,
    /// Error represents a recoverable error, report and continue the Client.
    Error(String),
    /// Fatal represents an unrecoverable error, report and quit the Client.
    Fatal(String),
}

/// Flag is the schema for a single flag of a Command. It directs the Client
/// about how to parse its arguments so that it can be used by its Command.
#[derive(Clone, Copy)]
pub enum Flag {
    /// A Boolean Flag takes no arguments.
    Boolean,
    /// A Single Flag takes a single argument.
    Single,
    /// An Array Flag takes a constant amount of arguments.
    Array(usize),
    /// A Variadic Flags takes all the remaining arguments.
    Variadic,
}

impl Flag {
    /// collect returns the number of arguments that should be collected from the
    /// given list of arguments for the current flags. The caller must check if the
    /// list has enough arguments to collect from.
    fn collect(&self, from: &[&str]) -> usize {
        match self {
            Flag::Boolean => 0,
            Flag::Single => 1,
            Flag::Array(n) => *n,
            Flag::Variadic => from.len(),
        }
    }
}

/// FlagValues stores the arguments provided to each Flag during a single
/// invocation of the parent Command. It is provided to the run function.
#[derive(Default)]
pub struct FlagValues {
    bool_flags: HashSet<String>,
    sing_flags: HashMap<String, String>,
    arry_flags: HashMap<String, Vec<String>>,
}

impl FlagValues {
    /// is_set checks if the Flag with the given name was provided in the invocation.
    pub fn is_set(&self, flag: &str) -> bool {
        self.bool_flags.contains(flag)
            | self.sing_flags.contains_key(flag)
            | self.arry_flags.contains_key(flag)
    }

    /// get_single returns the value of the given [`Flag::Single`].
    pub fn get_single(&self, flag: &str) -> Option<String> {
        self.sing_flags.get(flag).map(|s| s.to_owned())
    }

    /// get_array returns the value of the given [`Flag::Array`] or [`Flag::Variadic`].
    pub fn get_array(&self, flag: &str) -> Option<Vec<String>> {
        self.arry_flags.get(flag).map(|s| s.to_owned())
    }
}

impl FlagValues {
    fn insert(&mut self, name: &str, flag: Flag, value: &[&str]) {
        let name = name.to_string();
        let value = Vec::from_iter(value.iter().map(|s| s.to_string()));
        match flag {
            Flag::Boolean => {
                self.bool_flags.insert(name);
            }
            Flag::Single => {
                self.sing_flags.insert(name, value[0].clone());
            }
            Flag::Array(_) | Flag::Variadic => {
                self.arry_flags.insert(name, value);
            }
        };
    }
}
