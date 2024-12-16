use std::str::FromStr;

use tetka::games::isolation;
use tetka::uxi::Command;

use super::Context;

pub fn d() -> Command<Context> {
    Command::new(|bundle| {
        let ctx = bundle.lock();
        println!("{}", ctx.position);

        Ok(())
    })
}

pub fn uginewgame() -> Command<Context> {
    Command::new(|bundle| {
        let mut ctx = bundle.lock();
        ctx.position = isolation::Position::from_str(
            "--------/--------/p-------/-------P/--------/-------- w 1",
        )?;

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
