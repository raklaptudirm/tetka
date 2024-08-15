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
use std::error::Error;
use std::fmt;
use std::thread;

use crate::context::{new_bundle, GuardedBundledCtx};
use crate::{flag, Bundle, Flag};

/// Command represents a runnable UAI command. It contains all the metadata
/// needed to parse and verify a Command request from the GUI for a Command, and
/// to run that Command with the current context and the provided flag values.
/// `T` is the context type of the Client, while `E` is the error type. `E`
/// must implement the [`RunError`] trait to be usable.
///
/// A Command's schema is composed of its name, run function, flag schema, and
/// whether it is run in parallel. When a Command is invoked by a GUI, the
/// invocation starts with its name followed by any number of flags from its
/// flag schema. See the documentation of [`Flag`] for more details.
pub struct Command<T: Send> {
    /// run_fn is the function used to run this Command.
    pub run_fn: RunFn<T>,
    /// flags is the schema of the Flags this Command accepts.
    pub flags: HashMap<String, Flag>,
    /// parallel says whether to run this command in a separate thread.
    pub parallel: bool,
}

impl<T: Send + 'static> Command<T> {
    /// run runs the current Command with the given context and flag values.
    /// A new thread is spawned and detached to run parallel Commands. It returns
    /// the error returned by the Command's execution, or [`Ok`] for parallel.
    pub fn run<const PARALLEL: bool>(
        &self,
        context: &GuardedBundledCtx<T>,
        flags: flag::Values,
    ) -> CmdResult {
        // Clone values which might be moved by spawning a new thread.
        let context = new_bundle(context, flags);
        let func = self.run_fn;

        if PARALLEL && self.parallel {
            // If the Command is supposed to be run in parallel, spawn a new
            // thread and detach it for its execution. Syncing with the thread
            // should be handled by the user using the context.
            thread::spawn(move || func(context));
            return Ok(());
        }

        // Run the synchronous Command and return its error.
        func(context)
    }
}

impl<T: Send> Command<T> {
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
    /// let cmd: Command<T> =
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
    pub fn new(func: RunFn<T>) -> Command<T> {
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

/// `RunFn<T>` represents the run function of a Command. This function is called
/// with the context ([`Bundle<T>`]) and the flag values whenever the Command
/// is invoked. It returns a `CmdResult` which is then handled by the Client.
pub type RunFn<T> = fn(Bundle<T>) -> CmdResult;

/// CmdResult is the [Result] type returned by a [run function](RunFn). It is
/// a shorthand for `Result<(), RunError>`.
pub type CmdResult = Result<(), RunError>;

/// `quit!()` resolves to a [`Err(~RunError::Quit)`](RunError::Quit) kind of
/// error, and thus can be called by itself inside a Command to instruct the
/// Client to quit itself and stop executing commands.
#[macro_export]
macro_rules! quit {
    () => {
        Err(RunError::Quit)
    };
}

/// `error!()` resolves to a [`Err(~RunError::Error)`](RunError::Error) kind of
/// error, and thus can be called by itself inside a Command to exit the Command
/// with the given error. It supports the same arguments as the [`format!`] macro.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        Err(RunError::Error(format!($($arg)*)))
    };
}

/// `fatal!()` resolves to a [`Err(~RunError::Fatal)`](RunError::Fatal) kind of
/// error, and thus can be called by itself inside a Command to exit the Command
/// with the given error and to quit the Client. It is similar to the [`error!`]
/// macro and supports the same arguments.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {
        Err(RunError::Fatal(format!($($arg)*)))
    };
}

/// RunError is the error that is used internally in Client. All user errors
/// must support conversion into this type so that the Client can handle them.
#[derive(Debug, Clone)]
pub enum RunError {
    /// Quit directs the Client to quit itself, without reporting any errors.
    Quit,
    /// Error represents a recoverable error, report and continue the Client.
    Error(String),
    /// Fatal represents an unrecoverable error, report and quit the Client.
    Fatal(String),
}

impl<E> From<E> for RunError
where
    E: Error + Send + Sync + 'static,
{
    /// `From<E>` is implemented on [RunError] for all [errors][Error], allowing the
    /// usage of the `?` operator inside the body of a Command's run function to
    /// make error handling much easier. Errors are mapped to [`RunError::Error`].
    fn from(value: E) -> Self {
        RunError::Error(value.to_string())
    }
}

impl RunError {
    /// should_quit checks if the current error requires the Client to quit.
    pub fn should_quit(&self) -> bool {
        // Except Error, all other variants cause the Client to quit.
        !matches!(self, Self::Error(_err))
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Quit => Ok(()),
            RunError::Error(o_o) => write!(f, "info error {}", o_o),
            RunError::Fatal(o_o) => write!(f, "info error {}", o_o),
        }
    }
}
