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
use std::sync::{Arc, Mutex, MutexGuard};

use crate::{flag, parameter, BundledCtx, Parameter};

/// Bundle is a packet containing all the relevant context necessary for a
/// [Command](crate::Command) invocation. It provides access to the values of
/// the flags provided to the command during invocation, the user specific
/// context, and the inbuilt context for use in a Command's run function. A
/// given Bundle is tied to a Command's invocation and can't be used outside
/// that context.
pub struct Bundle<T: Send> {
    context: GuardedBundledCtx<T>,
    flags: flag::Values,
}

/// new creates a new [`Bundle<T>`] with the given [`BundledCtx`] and [`FlagValues`].
pub fn new_bundle<T: Send>(context: &GuardedBundledCtx<T>, flags: flag::Values) -> Bundle<T> {
    let context = Arc::clone(context);
    Bundle { context, flags }
}

impl<T: Send> Bundle<T> {
    /// lock locks the internal mutex of the Bundle and returns a mutex-locked
    /// [`BundledCtx`] which allows access to the user provided and inbuilt contexts
    /// stored in the [Client](crate::Client). The mutex can be unlocked by calling
    /// the `drop` function on the variable storing the mutex guard.
    /// ```rust,ignore
    /// // bundle: Bundle<T>
    /// let context = bundle.lock(); // Locking the mutex.
    /// drop(context);               // Unlocking the mutex.
    /// ```
    /// You can lock the mutex again by calling the `lock` function on the Bundle.
    /// Remember to unlock the mutex when not in use in parallel or long-running
    /// Commands so that other Commands don't get stuck trying to access the
    /// contexts. In other cases, the mutex is unlocked when the Command ends.
    pub fn lock(&self) -> MutexGuard<'_, BundledCtx<T>> {
        self.context.lock().unwrap()
    }

    /// is_flag_set checks if a flag with the given name was provided to the Command
    /// during invocation. It works for all types of flags, not just boolean ones.
    pub fn is_flag_set(&self, name: &str) -> bool {
        self.flags.is_set(name)
    }

    /// get_single_flag gets the value provided to a [single](crate::Flag::Single)
    /// flag during the Command's invocation. It returns [`None`] if the flag was
    /// not set during invocation.
    pub fn get_single_flag(&self, name: &str) -> Option<String> {
        self.flags.get_single(name)
    }

    /// get_single_flag gets the value provided to an [array](crate::Flag::Array)
    /// or a [variadic](crate::Flag::Variadic) flag during the Command's invocation.
    /// It returns [`None`] if the flag was not set during invocation.
    pub fn get_array_flag(&self, name: &str) -> Option<Vec<String>> {
        self.flags.get_array(name)
    }
}

/// A GuardedBundledCtx is a [BundledCtx] with a reference-counted mutex guard,
/// which allows it to be used by multiple Commands concurrently without issues.
pub type GuardedBundledCtx<T> = Arc<Mutex<BundledCtx<T>>>;

#[derive(Clone)]
pub struct Context {
    /// The name of this Client's engine.
    pub engine: String,
    /// The author of this Client's engine.
    pub author: String,

    /// The UXI protocol supported by this Client.
    pub protocol: String,
    /// The currently selected protocol. It can have the values "" for when no uxi
    /// command has been received, "ugi", or <protocol> for those protocols.
    pub selected_protocol: String,

    /// Schema of the options supported by this Client.
    pub options: HashMap<String, Parameter>,
    /// Values of the options supported by this Client.
    pub option_values: parameter::Values,
}

impl Context {
    /// setoption sets the value of the given option to the given value.
    pub fn setoption(&mut self, name: &str, value: &str) -> Result<(), String> {
        let option = self.options.get(name);
        if option.is_none() {
            return Err(format!("unknown option \"{}\"", name));
        }

        self.option_values
            .insert(name.to_owned(), option.unwrap(), value)
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            engine: "Nameless v0.0.0".to_string(),
            author: "Anonymous".to_string(),
            protocol: "".to_string(),
            selected_protocol: "".to_string(),
            options: HashMap::new(),
            option_values: Default::default(),
        }
    }
}
