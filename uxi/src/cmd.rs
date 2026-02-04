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

use crate::bundles::{Bundle, GuardedBundledCtx};
use crate::{flag, Flag};

/// Command represents a runnable UXI command.
///
/// It contains all the metadata needed to parse and verify a Command request
/// from the GUI for a Command, and to run that Command with the current context
/// and the provided flag values. The `C` type parameter is the same type as the
/// context used by Client to maintain its state across different commands.
///
/// A new Command can be created using the [`Command::new`] function, and its
/// builder methods can be used for defining other properties of the command.
pub struct Command<C: Send> {
    /// run_fn is the function used to run this Command.
    run_fn: RunFn<C>,
    /// flags is the schema of the Flags this Command accepts.
    pub(crate) flags: HashMap<String, Flag>,
    /// parallel says whether to run this command in a separate thread.
    parallel: bool,
}

impl<C: Send + 'static> Command<C> {
    /// run runs the current Command with the given context and flag values.
    /// A new thread is spawned and detached to run parallel Commands. It returns
    /// the error returned by the Command's execution, or [`Ok`] for parallel.
    pub(crate) fn run<const PARALLEL: bool>(
        &self,
        context: &GuardedBundledCtx<C>,
        flags: flag::Values,
    ) -> CmdResult {
        // Clone values which might be moved by spawning a new thread.
        let context = Bundle::new(context, flags);
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

impl<C: Send> Command<C> {
    /// new creates a new Command with the given run function.
    ///
    /// A Command has a body function along with two main properties, its flag
    /// schema and whether it blocks the main command loop. By default the flag
    /// schema is empty the the command blocks the main command loop. The body
    /// function needs to be passed as the argument of type [`RunFn<Context>`].
    ///
    /// Further configuration of the Command can be done using the builder style
    /// methods provided on Command, like [`Command::flag`] and
    /// [`Command::parallelize`]. These methods take the ownership of the given
    /// Command, make the necessary changes and then return it.
    /// ```
    /// # use uxi::{Command, Flag};
    /// # type Context = u64;
    /// let cmd: Command<Context> =
    ///     // new invocation to create a Command. In this example, a very
    ///     // simple run function which returns `Ok(())` is provided.
    ///     Command::new(|_ctx| Ok(()))
    ///         // Add flags to the Command's flag schema.
    ///         .flag("flag1", Flag::Boolean)
    ///         .flag("flag2", Flag::Single)
    ///         .flag("flag3", Flag::Array(10))
    ///         .flag("flag4", Flag::Variadic)
    ///         // Make the command run in parallel.
    ///         .parallelize(true);
    /// ```
    pub fn new(func: RunFn<C>) -> Command<C> {
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

/// `RunFn<C>` represents the body function of a Command.
///
/// The generic argument `C` is the same as the `C` in [`Client`](super::Client).
///
/// It takes a single argument of type [`Bundle<Context>`]. A bundle, as the
/// name suggests, bundles the command's flag values, the current option values,
/// the user specified internal context (`Context`), and the uxi provided
/// internal context into a single value.
///
/// A value of type [`CmdResult`] is returned, which is a [`Result`] with
/// [`RunError`] as the error type.
pub type RunFn<C> = fn(Bundle<C>) -> CmdResult;

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

/// RunError is the error type returned when running a Command.
///
/// Its a powerful dynamic error type which supports conversion from most error
/// types which allows for idiomatic error handling with rust language features
/// like the `?` operator.
///
/// A blanket implementation of [`Into<RunError>`] is available for all types
/// which are [`Error`], [`Send`], [`Sync`], and `'static`.
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
