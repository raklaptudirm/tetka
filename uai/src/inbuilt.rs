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

use crate::{quit, RunErrorType};
use lazy_static::lazy_static;

pub type Command = crate::Command<Context, RunErrorType>;

lazy_static! {
    pub static ref COMMANDS: HashMap<String, Command> = HashMap::from(
        [
            ("quit", Command::new(|_ctx, _flags| quit!())),
            (
                "isready",
                Command::new(|_ctx, _flags| {
                    println!("readyok");
                    Ok(())
                })
            ),
            (
                "uai",
                Command::new(|ctx, _flags| {
                    let ctx = ctx.lock().unwrap();

                    println!("id name {}", ctx.engine);
                    println!("id author {}", ctx.author);
                    println!();
                    println!("uaiok");

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
}

impl Default for Context {
    fn default() -> Self {
        Context {
            engine: "Nameless v0.0.0".to_string(),
            author: "Anonymous".to_string(),
        }
    }
}
