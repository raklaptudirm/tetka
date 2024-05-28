use std::collections::HashMap;
use std::fmt;

/// Parameter is the schema for the Client's options.
#[derive(Clone)]
pub enum Parameter {
    /// Check represents a checkbox parameter which can be true or false.
    ///
    /// Its first field contains the default value for the checkbox.
    Check(bool),

    /// String represents a string parameter which can have any string value.
    ///
    /// Its first field contains the default String value for the parameter.
    String(String),

    /// Spin represents a spin wheel which can be an integer in a certain range.
    ///
    /// The first argument is the default value, the second argument is the minimum
    /// value, and the third argument is the maximum value of this parameter. The
    /// minimum and maximum bounds are inclusive and and the default must be in
    /// the range defined by them.
    Spin(i64, i64, i64),

    /// Combo represents a combo box which can have the value of one of the
    /// predefined strings.
    ///
    /// Its first argument is the default value while the second argument is the
    /// list of predefined strings. The default value must be included in the list.
    Combo(String, Vec<String>),
}

#[derive(Clone, Default)]
pub struct Values {
    checks: HashMap<String, bool>,
    strings: HashMap<String, String>,
    numbers: HashMap<String, i64>,
}

impl Values {
    pub fn get_check(&self, name: &str) -> Option<bool> {
        self.checks.get(name).copied()
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        self.strings.get(name).cloned()
    }

    pub fn get_spin(&self, name: &str) -> Option<i64> {
        self.numbers.get(name).copied()
    }
}

impl Values {
    pub fn insert(
        &mut self,
        name: String,
        option: &Parameter,
        value_str: &str,
    ) -> Result<(), String> {
        match option {
            Parameter::Check(_) => {
                let value = value_str.parse();
                if value.is_err() {
                    return Err(format!(
                        "option {}: expected boolean, received {}",
                        name, value_str
                    ));
                }
                self.checks.insert(name, value.unwrap());
            }
            Parameter::String(_) => {
                self.strings.insert(name, value_str.to_owned());
            }
            Parameter::Spin(_, min, max) => {
                let value = value_str.parse();
                if value.is_err() {
                    return Err(format!(
                        "option {}: expected a number, received {}",
                        name, value_str
                    ));
                }

                let value = value.unwrap();
                if value < *min || value > *max {
                    return Err(format!(
                        "option {}: expected a number between {} and {} (inclusive), received {}",
                        name, min, max, value_str
                    ));
                }
                self.numbers.insert(name, value);
            }
            Parameter::Combo(_, strings) => {
                let value = value_str.to_owned();
                if strings.contains(&value) {
                    return Err(format!(
                        "option {}: {} is not one of the combo strings",
                        name, value
                    ));
                }
                self.strings.insert(name, value);
            }
        };

        Ok(())
    }

    pub fn insert_default(&mut self, name: String, option: &Parameter) {
        match option {
            Parameter::Check(default) => {
                self.checks.insert(name, *default);
            }
            Parameter::String(default) => {
                self.strings.insert(name, default.clone());
            }
            Parameter::Spin(default, _, _) => {
                self.numbers.insert(name, *default);
            }
            Parameter::Combo(default, _) => {
                self.strings.insert(name, default.clone());
            }
        };
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Check(default) => write!(f, "check default {}", default),
            Parameter::String(default) => write!(f, "string default {}", default),
            Parameter::Spin(default, min, max) => {
                write!(f, "spin default {} min {} max {}", default, min, max)
            }
            Parameter::Combo(default, combos) => {
                write!(f, "combo default {}", default)?;
                for combo in combos {
                    write!(f, " var {}", combo)?
                }
                Ok(())
            }
        }
    }
}
