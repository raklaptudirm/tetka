use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::{fmt, thread};

pub struct Client<T: Send> {
    commands: HashMap<String, Command<T>>,
}

impl<T: Send + 'static> Client<T> {
    pub fn add_command(&mut self, name: &str, cmd: Command<T>) {
        self.commands.insert(name.to_string(), cmd);
    }

    pub fn start(&mut self, context: T) {
        let stdin = io::stdin();
        let context = Arc::new(Mutex::new(context));

        'reading: for line in stdin.lock().lines() {
            let line = line.unwrap();
            let parts = line.split(' ').collect::<Vec<&str>>();

            let cmd_name = parts[0];
            let mut args = &parts[1..];

            let cmd = self.commands.get(cmd_name);

            if cmd.is_none() {
                println!("info error command {} not found", cmd_name);
                continue 'reading;
            }

            let cmd = cmd.unwrap();
            let mut flags: Flags = Default::default();

            while !args.is_empty() {
                let flag_name = args[0];
                args = &args[1..];
                let flag = cmd.flags.get(flag_name);

                if flag.is_none() {
                    println!("info error flag {} not found", flag_name);
                    continue 'reading;
                }

                let flag = flag.unwrap();
                let yank = flag.collect(args);

                let collected = &args[..yank];
                flags.insert(flag_name, *flag, collected);
                args = &args[yank..];
            }

            match cmd.run(&context, flags) {
                RunError::None => (),
                RunError::Quit => break,
                RunError::Error(o_o) => println!("{}", o_o),
                RunError::Fatal(o_o) => {
                    println!("{}", o_o);
                    break;
                }
            };
        }
    }
}

impl<T: Sync + Send> Default for Client<T> {
    fn default() -> Self {
        Client {
            commands: HashMap::new(),
        }
    }
}

type RunFn<T> = fn(Arc<Mutex<T>>, Flags) -> Result<(), RunError>;

pub struct Command<T> {
    pub run_fn: RunFn<T>,
    pub flags: HashMap<String, Flag>,
    pub parallel: bool,
}

impl<T: Send + 'static> Command<T> {
    pub fn new(func: RunFn<T>) -> Command<T> {
        Command {
            run_fn: func,
            flags: Default::default(),
            parallel: false,
        }
    }

    pub fn run(&self, context: &Arc<Mutex<T>>, flags: Flags) -> RunError {
        let context = Arc::clone(context);
        let func = self.run_fn;

        if self.parallel {
            thread::spawn(move || func(context, flags));
            return RunError::None;
        }

        match (self.run_fn)(context, flags) {
            Ok(_) => RunError::None,
            Err(err) => err,
        }
    }

    pub fn add_flag(&mut self, name: &str, flag: Flag) {
        self.flags.insert(name.to_string(), flag);
    }
}

pub enum RunError {
    None,
    Quit,
    Error(String),
    Fatal(String),
}

impl From<RunError> for Result<(), RunError> {
    fn from(value: RunError) -> Self {
        Err(value)
    }
}

impl From<&dyn fmt::Debug> for RunError {
    fn from(value: &dyn fmt::Debug) -> Self {
        Self::Error(format!("{:?}", value))
    }
}

#[derive(Clone, Copy)]
pub enum Flag {
    Boolean,
    Single,
    Array(usize),
    Variadic,
}

impl Flag {
    pub fn collect(&self, from: &[&str]) -> usize {
        match self {
            Flag::Boolean => 0,
            Flag::Single => 1,
            Flag::Array(n) => *n,
            Flag::Variadic => from.len(),
        }
    }
}

#[derive(Default)]
pub struct Flags {
    bool_flags: HashSet<String>,
    sing_flags: HashMap<String, String>,
    arry_flags: HashMap<String, Vec<String>>,
}

impl Flags {
    fn insert(&mut self, name: &str, flag: Flag, value: &[&str]) {
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

    pub fn is_set(&self, flag: &str) -> bool {
        self.bool_flags.contains(flag)
            | self.sing_flags.contains_key(flag)
            | self.arry_flags.contains_key(flag)
    }

    pub fn get_single(&self, flag: &str) -> String {
        self.sing_flags.get(flag).unwrap().to_owned()
    }

    pub fn get_array(&self, flag: &str) -> Vec<String> {
        self.arry_flags.get(flag).unwrap().to_owned()
    }
}
