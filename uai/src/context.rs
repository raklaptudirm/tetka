use std::sync::{Arc, Mutex, MutexGuard};

use crate::{inbuilt, FlagValues, Number};

pub struct Bundle<T: Send> {
    context: Arc<Mutex<BundledCtx<T>>>,
    flags: FlagValues,
}

impl<T: Send> Bundle<T> {
    pub fn new(context: &Arc<Mutex<BundledCtx<T>>>, flags: FlagValues) -> Bundle<T> {
        let context = Arc::clone(context);
        Bundle { context, flags }
    }

    pub fn lock(&self) -> MutexGuard<'_, BundledCtx<T>> {
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

pub struct BundledCtx<T: Send> {
    pub user: T,
    pub client: inbuilt::Context,
}

impl<T: Send> BundledCtx<T> {
    pub fn get_check_option(&self, name: &str) -> Option<bool> {
        self.client.option_values.get_check(name)
    }

    pub fn get_string_option(&self, name: &str) -> Option<String> {
        self.client.option_values.get_string(name)
    }

    pub fn get_spin_option(&self, name: &str) -> Option<Number> {
        self.client.option_values.get_spin(name)
    }
}
