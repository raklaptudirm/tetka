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

use crate::{error, quit, Flag, Parameter, ParameterValues, RunErrorType};
use lazy_static::lazy_static;

pub type Command = crate::Command<Context, RunErrorType>;

lazy_static! {
    pub static ref COMMANDS: HashMap<String, Command> = HashMap::from(
        [
            ("quit", Command::new(|_ctx| quit!())),
            (
                "isready",
                Command::new(|_ctx| {
                    println!("readyok");
                    Ok(())
                })
            ),
            (
                "uai",
                Command::new(|ctx| {
                    let ctx = ctx.lock();

                    println!("id name {}", ctx.engine);
                    println!("id author {}", ctx.author);
                    println!();
                    if !ctx.options.is_empty() {
                        for (name, option) in ctx.options.clone() {
                            println!("option name {} type {}", name, option);
                        }

                        println!();
                    }
                    println!("uaiok");

                    Ok(())
                })
            ),
            (
                "setoption",
                Command::new(|ctx| {
                    let name = ctx.get_single_flag("name");
                    let value = ctx.get_array_flag("value");

                    if name.is_none() || value.is_none() {
                        return error!("expected \"name\" and \"value\" flags");
                    }

                    let name = name.unwrap();
                    let value = value.unwrap().join(" ");

                    let mut ctx = ctx.lock();

                    ctx.setoption(&name, &value).map_err(RunErrorType::Error)
                })
                .flag("name", Flag::Single)
                .flag("value", Flag::Variadic)
            ),
            (
                "options",
                Command::new(|ctx| {
                    let ctx = ctx.lock();

                    for (name, option) in ctx.options.clone() {
                        print!("option name {} value ", name);
                        match option {
                            Parameter::Check(_) => {
                                println!("{}", ctx.option_values.get_check(&name).unwrap())
                            }
                            Parameter::String(_) | Parameter::Combo(_, _) => {
                                println!("{}", ctx.option_values.get_string(&name).unwrap())
                            }
                            Parameter::Spin(_, _, _) => {
                                println!("{}", ctx.option_values.get_spin(&name).unwrap())
                            }
                        }
                    }

                    Ok(())
                })
            )
        ]
        .map(|a| {
            let (b, c) = a;
            (b.to_owned(), c)
        })
    );
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
