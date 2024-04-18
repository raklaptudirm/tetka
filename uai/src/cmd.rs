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
use std::sync::{Arc, Mutex};
use std::thread;

use super::{Flag, FlagValues};

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

/// RunFn<T, E> represents the run function of a Command. This function is called
/// with the context (`Arc<Mutex<T>>`) and the flag values ([`FlagValues`]) whenever
/// the Command is to be executed. It returns a `Result<(), E>` where `E` implements
/// [`RunError`] and is the error type for the [Client].
pub type RunFn<T, E> = fn(Arc<Mutex<T>>, FlagValues) -> Result<(), E>;

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
        Err(RunErrorType::Quit.into())
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
            Err(RunErrorType::Error(format!($($arg)*)).into())
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
            RunErrorType::Error(format!($($arg)*)).into()
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
        Err(RunErrorType::Fatal(format!($($arg)*)).into())
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
