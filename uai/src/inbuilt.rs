use crate::{quit, Command, RunError, RunErrorType};

pub fn quit<T: Send, E: RunError>() -> Command<T, E> {
    Command::new(|_ctx, _flag| quit!())
}

pub fn isready<T: Send, E: RunError>() -> Command<T, E> {
    Command::new(|_ctx, _flag| {
        println!("readyok");
        Ok(())
    })
}
