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
use std::sync::{Arc, Mutex};

use crate::{context::Context, GuardedBundledCtx, Number};

/// A BundledCtx bundles the user-provided context `C` and the inbuilt context
/// into a single type of ease of mutex guarding for concurrency.
pub struct BundledCtx<T: Send> {
    user: T,
    client: Context,
}

pub fn new_guarded_ctx<T: Send>(user: T, client: Context) -> GuardedBundledCtx<T> {
    Arc::new(Mutex::new(BundledCtx { user, client }))
}

impl<T: Send> BundledCtx<T> {
    pub fn protocol(&self) -> String {
        self.client.selected_protocol.clone()
    }

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

impl<T: Send> Deref for BundledCtx<T> {
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

pub mod commands {
    use crate::{context::Context, error, quit, Command, Flag, Parameter, RunError};

    pub fn quit<C: Send>() -> Command<C> {
        Command::new(|_ctx| quit!())
    }

    pub fn isready<C: Send>() -> Command<C> {
        Command::new(|_ctx| {
            println!("readyok");
            Ok(())
        })
    }

    pub fn uxi<C: Send>() -> Command<C> {
        #[allow(clippy::assigning_clones)]
        Command::new(|ctx| {
            let mut ctx = ctx.lock();

            print_protocol_info(&ctx.client);
            println!("{}ok", ctx.client.protocol);

            ctx.client.selected_protocol = ctx.client.protocol.clone();

            Ok(())
        })
    }

    pub fn ugi<C: Send>() -> Command<C> {
        Command::new(|ctx| {
            let mut ctx = ctx.lock();

            print_protocol_info(&ctx.client);
            println!("ugiok");

            ctx.client.selected_protocol = "ugi".to_string();

            Ok(())
        })
    }

    fn print_protocol_info(ctx: &Context) {
        println!("id name {}", ctx.engine);
        println!("id author {}", ctx.author);
        println!();
        if !ctx.options.is_empty() {
            for (name, option) in ctx.options.clone() {
                println!("option name {} type {}", name, option);
            }

            println!();
        }
    }

    pub fn setoption<C: Send>() -> Command<C> {
        Command::new(|ctx| {
            let name = ctx.get_single_flag("name");
            let value = ctx.get_array_flag("value");

            if name.is_none() || value.is_none() {
                return error!("expected \"name\" and \"value\" flags");
            }

            let name = name.unwrap();
            let value = value.unwrap().join(" ");

            let mut ctx = ctx.lock();

            ctx.client.setoption(&name, &value).map_err(RunError::Error)
        })
        .flag("name", Flag::Single)
        .flag("value", Flag::Variadic)
    }

    pub fn options<C: Send>() -> Command<C> {
        Command::new(|ctx| {
            let ctx = ctx.lock();

            for (name, option) in ctx.client.options.clone() {
                print!("option name {} value ", name);
                match option {
                    Parameter::Check(_) => {
                        println!("{}", ctx.client.option_values.get_check(&name).unwrap())
                    }
                    Parameter::String(_) | Parameter::Combo(_, _) => {
                        println!("{}", ctx.client.option_values.get_string(&name).unwrap())
                    }
                    Parameter::Spin(_, _, _) => {
                        println!("{}", ctx.client.option_values.get_spin(&name).unwrap())
                    }
                }
            }

            Ok(())
        })
    }
}
