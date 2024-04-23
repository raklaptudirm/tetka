use std::sync::{Arc, Mutex, MutexGuard};

use crate::FlagValues;

pub struct Context<T: Send> {
    context: Arc<Mutex<T>>,
    flags: FlagValues,
}

impl<T: Send> Context<T> {
    pub fn new(context: &Arc<Mutex<T>>, flags: FlagValues) -> Context<T> {
        let context = Arc::clone(context);
        Context { context, flags }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.context.lock().unwrap()
    }

    pub fn is_flag_set(&self, name: &str) -> bool {
        self.flags.is_set(name)
    }

    pub fn get_single_flag(&self, name: &str) -> Option<String> {
        self.flags.get_single(name)
    }

    pub fn get_array_flag(&self, name: &str) -> Option<Vec<String>> {
        self.flags.get_array(name)
    }
}
