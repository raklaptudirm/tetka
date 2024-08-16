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

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::flag;
use crate::inbuilt::Context;

// -------------------------------------------------- //
// Dependency Graph of the Various Bundle structures: //
// -------------------------------------------------- //
// Bundle(Encapsulation) --locking--vv                //
//      /                  MutexGuard<'_, BundledCtx> //
// Flag Values            \                           //
//         GuardedBundledCtx(Mutex-guard)             //
//                          \                         //
//               BundledCtx(Encapsulation)            //
//                     /            \                 //
//            User::Context   Inbuilt::Context        //
// -------------------------------------------------- //

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

/// new creates a new [`Bundle`] with the given [`BundledCtx`] and [`FlagValues`].
pub(crate) fn new_bundle<T: Send>(
    context: &GuardedBundledCtx<T>,
    flags: flag::Values,
) -> Bundle<T> {
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
pub(crate) type GuardedBundledCtx<C> = Arc<Mutex<BundledCtx<C>>>;

/// A BundledCtx bundles the user-provided context `C` and the inbuilt context
/// into a single type of ease of mutex guarding for concurrency. It provides
/// methods which allow Commands to retrieve information from those contexts.
pub struct BundledCtx<C: Send> {
    user: C,
    pub(crate) client: Context,
}

/// new_guarded_ctx created a new [GuardedBundledCtx] from the given user and
/// client contexts.
pub(crate) fn new_guarded_ctx<C: Send>(
    user: C,
    client: Context,
) -> GuardedBundledCtx<C> {
    Arc::new(Mutex::new(BundledCtx { user, client }))
}

impl<T: Send> BundledCtx<T> {
    /// protocol returns the last protocol command which was issues to the Client.
    /// It returns "" if no protocol command has been issued to the engine till now.
    pub fn protocol(&self) -> String {
        self.client.selected_protocol.clone()
    }

    /// get_check_option returns the value of a check option with the given name.
    pub fn get_check_option(&self, name: &str) -> Option<bool> {
        self.client.option_values.get_check(name)
    }

    /// get_string_option returns the value of a combo/string option with the given
    /// name.
    pub fn get_string_option(&self, name: &str) -> Option<String> {
        self.client.option_values.get_string(name)
    }

    /// get_spin_option returns the value of a spin option with the given name.
    pub fn get_spin_option(&self, name: &str) -> Option<i64> {
        self.client.option_values.get_spin(name)
    }
}

impl<T: Send> Deref for BundledCtx<T> {
    /// A BundledCtx can be dereferenced into the user's context and freely
    /// manipulated. This is because both Deref and DerefMut are implemented.
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

impl<T: Send> DerefMut for BundledCtx<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.user
    }
}
