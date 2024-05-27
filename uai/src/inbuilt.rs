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

use crate::{parameter::ParameterValues, GuardedBundledCtx, Number, Parameter};

/// A BundledCtx bundles the user-provided context `C` and the inbuilt context
/// ([`Context`]) into a single type of ease of mutex guarding.
pub struct BundledCtx<T: Send> {
    pub user: T,
    client: Context,
}

pub fn new_guarded_ctx<T: Send>(user: T, client: Context) -> GuardedBundledCtx<T> {
    Arc::new(Mutex::new(BundledCtx { user, client }))
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

pub mod commands {
    use crate::{error, quit, Command, Flag, Parameter, RunError};

    pub fn quit<C: Send>() -> Command<C> {
        Command::new(|_ctx| quit!())
    }

    pub fn isready<C: Send>() -> Command<C> {
        Command::new(|_ctx| {
            println!("readyok");
            Ok(())
        })
    }

    pub fn uai<C: Send>() -> Command<C> {
        Command::new(|ctx| {
            let ctx = ctx.lock();

            println!("id name {}", ctx.client.engine);
            println!("id author {}", ctx.client.author);
            println!();
            if !ctx.client.options.is_empty() {
                for (name, option) in ctx.client.options.clone() {
                    println!("option name {} type {}", name, option);
                }

                println!();
            }
            println!("uaiok");

            Ok(())
        })
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

#[derive(Clone)]
pub struct Context {
    pub engine: String,
    pub author: String,

    pub options: HashMap<String, Parameter>,
    pub option_values: ParameterValues,
}

impl Context {
    fn setoption(&mut self, name: &str, value: &str) -> Result<(), String> {
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
            options: HashMap::new(),
            option_values: Default::default(),
        }
    }
}
