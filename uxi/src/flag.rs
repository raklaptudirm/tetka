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

use std::collections::{HashMap, HashSet};

/// Flag is the schema for a single flag of a Command. It directs the Client
/// about how to parse its arguments so that it can be used by its Command.
/// When in a command invocation, a flag starts with its name followed by a
/// number of arguments depending upon its type.
#[derive(Clone, Copy)]
pub enum Flag {
    /// A Boolean Flag takes no arguments.
    Boolean,
    /// A Single Flag takes a single argument.
    Single,
    /// An Array Flag takes a constant amount of arguments.
    Array(usize),
    /// A Variadic Flags takes all the remaining arguments.
    Variadic,
}

impl Flag {
    /// collect returns the number of arguments that should be collected from the
    /// given list of arguments for the current flags. The caller must check if the
    /// list has enough arguments to collect from.
    pub fn collect(&self, from: &[&str]) -> usize {
        match self {
            Flag::Boolean => 0,
            Flag::Single => 1,
            Flag::Array(n) => *n,
            Flag::Variadic => from.len(),
        }
    }
}

/// FlagValues stores the arguments provided to each Flag during a single
/// invocation of the parent Command. It is provided to the run function.
#[derive(Default)]
pub struct Values {
    bool_flags: HashSet<String>,
    sing_flags: HashMap<String, String>,
    arry_flags: HashMap<String, Vec<String>>,
}

impl Values {
    /// is_set checks if the Flag with the given name was provided in the invocation.
    pub fn is_set(&self, flag: &str) -> bool {
        self.bool_flags.contains(flag)
            | self.sing_flags.contains_key(flag)
            | self.arry_flags.contains_key(flag)
    }

    /// get_single returns the value of the given [`Flag::Single`].
    pub fn get_single(&self, flag: &str) -> Option<String> {
        self.sing_flags.get(flag).map(|s| s.to_owned())
    }

    /// get_array returns the value of the given [`Flag::Array`] or [`Flag::Variadic`].
    pub fn get_array(&self, flag: &str) -> Option<Vec<String>> {
        self.arry_flags.get(flag).map(|s| s.to_owned())
    }
}

impl Values {
    /// insert adds the given [Flag] with the given value to the current flag
    /// value set which will be provided to the Command's run function.
    pub fn insert(&mut self, name: &str, flag: Flag, value: &[&str]) {
        let name = name.to_string();
        let value = Vec::from_iter(value.iter().map(|s| s.to_string()));
        match flag {
            Flag::Boolean => {
                self.bool_flags.insert(name);
            }
            Flag::Single => {
                self.sing_flags.insert(name, value[0].clone());
            }
            Flag::Array(_) | Flag::Variadic => {
                self.arry_flags.insert(name, value);
            }
        };
    }
}

impl Values {
    /// parses converts the given arguments flags into a Flag [Values] value.
    pub fn parse(value: &[&str], flag_set: &HashMap<String, Flag>) -> Result<Self, String> {
        let mut flags: Self = Default::default();

        let mut args = value;

        // The arguments have the following format:
        // { flag_name { flag_arg... } ... }
        while !args.is_empty() {
            let flag_name = args[0]; // The first arg has to be a flag name.
            args = &args[1..]; // Remove the flag name from the rest of the args.

            // Try to find a flag with the given name.
            let flag = flag_set.get(flag_name);
            if flag.is_none() {
                // Flag not found, return error and continue.
                return Err(format!("info error flag {} not found", flag_name));
            }

            // The Option<Flag> in not None, so it can be safely unwrapped.
            let flag = flag.unwrap();

            // Find the number of arguments the Flag expects.
            let yank = flag.collect(args);

            // Check if args has the required number of arguments.
            if args.len() < yank {
                return Err(format!(
                    "info error flag {} expects {} arguments, found {}",
                    flag_name,
                    yank,
                    args.len(),
                ));
            }

            // Collect that number of arguments from the remaining args.
            let collected = &args[..yank];
            flags.insert(flag_name, *flag, collected);
            args = &args[yank..];
        }

        Ok(flags)
    }
}
