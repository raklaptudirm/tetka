use std::str::FromStr;

use ataxx::Position;
use uxi::Command;

use super::Context;

pub fn d() -> Command<Context> {
    Command::new(|bundle| {
        let ctx = bundle.lock();
        println!("{}", ctx.position);

        Ok(())
    })
}

pub fn uainewgame() -> Command<Context> {
    Command::new(|bundle| {
        let mut ctx = bundle.lock();
        ctx.position = Position::from_str("x5o/7/7/7/7/7/o5x x 0 1")?;

        Ok(())
    })
}

pub fn protocol() -> Command<Context> {
    Command::new(|bundle| {
        let ctx = bundle.lock();
        println!("current protocol: {}", ctx.protocol());
        Ok(())
    })
}
