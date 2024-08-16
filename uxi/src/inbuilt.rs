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

use crate::{parameter, Parameter};
use std::collections::HashMap;

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

/// The commands module contains functions which resolve into one of the inbuilt
/// Commands which come pre-registered with the Client.
pub mod commands {
    use crate::inbuilt::Context;
    use crate::{error, quit, Command, Flag, Parameter, RunError};

    /// quit resolves into the quit Command which quits the Client.
    pub fn quit<C: Send>() -> Command<C> {
        Command::new(|_ctx| quit!())
    }

    /// isready resolves into the isready command which is used to ping the Client
    /// and check for responsiveness, i.e. if its ready for the next Command.
    pub fn isready<C: Send>() -> Command<C> {
        Command::new(|_ctx| {
            println!("readyok");
            Ok(())
        })
    }

    /// uxi resolves into one of the uxi commands which respond with information
    /// about the Client and 'uxiok' to show support for that UXI protocol.
    pub fn uxi<C: Send>() -> Command<C> {
        #[allow(clippy::assigning_clones)]
        Command::new(|ctx| {
            let mut ctx = ctx.lock();

            print_client_info(&ctx.client);
            println!("{}ok", ctx.client.protocol);

            ctx.client.selected_protocol = ctx.client.protocol.clone();

            Ok(())
        })
    }

    /// ugi is similar to the uxi Command, just for the UGI protocol.
    pub fn ugi<C: Send>() -> Command<C> {
        Command::new(|ctx| {
            let mut ctx = ctx.lock();

            print_client_info(&ctx.client);
            println!("ugiok");

            ctx.client.selected_protocol = "ugi".to_string();

            Ok(())
        })
    }

    /// print_client_info prints information about the Client, which is reported in
    /// response to a uxi type protocol command.
    fn print_client_info(ctx: &Context) {
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

    /// setoption is the Command to set the values of the different
    /// [options](Parameter) supported by the Client.
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

    /// options lists the [options](Parameter) supported by the Client and their
    /// currently set values. This command is not part of the UXI standard.
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
