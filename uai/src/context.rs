use std::sync::{Arc, Mutex, MutexGuard};

use crate::{flag, BundledCtx};

/// Bundle is a packet containing all the relevant context necessary for a
/// [Command](crate::Command) invocation. It provides access to the values of
/// the flags provided to the command during invocation, the user specific
/// context, and the inbuilt [context](inbuilt::Context) for use in a Command's
/// body. A given Bundle is tied to a Command's invocation and can't be used
/// outside that context.
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
