# <samp> uxi </samp>

![Build Status](https://img.shields.io/github/actions/workflow/status/raklaptudirm/mexx/ci.yml) ![License](https://img.shields.io/crates/l/uxi) ![Crates.io](https://img.shields.io/crates/v/uxi
)

<samp>uxi</samp> is a package used to build UXI protocol compliant game engines easily. A [`Client`](https://docs.rs/uxi/latest/uxi/struct.Client.html) is the main representation of a game engine, refer to its documentation for more information. The commands which engine supports being sent to it from a GUI or other is represented as a [`Command`](https://docs.rs/uxi/latest/uxi/struct.Command.html), refer to its documentation for more details.

```rust
use uxi::*;

fn main() {
    // Setting up and starting the Client.
    Client::new()
        // Set engine protocol (Universal Ataxx Interface here).
        .protocol("uai")
        // Set engine details.
        .engine("Engine v0.0.0")
        .author("Rak Laptudirm")
        // Register engine options.
        .option("Hash", Parameter::Spin(16, 1, 8192))
        .option("Threads", Parameter::Spin(1, 1, 99))
        // Register the custom commands.
        .command("i", i())
        .command("d", d())
        // Start the Client so it can start running Commands.
        .start(Context { number: 0 });
}

// A custom user specified context stores information which persists across
// different Commands. In this case, we are storing a number which is used
// differently inside different Commands.
pub struct Context {
    pub number: i32,
}

// The command i increases the current value of the number by delta. Commands
// take a type parameter which specifies the type of the user defined Context.
pub fn i() -> Command<Context> {
    // The main part of a Command is its run function. The Bundle provided to the
    // run function provides access to the user specified context, flags, and other
    // things. Look at the documentation for more details.
    Command::new(|bundle: Bundle<Context>| {
        // Lock the mutex guarding the context bundle.
        let mut ctx = bundle.lock();

        // Get the value of the single flag named 'delta'.
        match bundle.get_single_flag("delta") {
            Some(delta) => {
                // Mutate the contents of the context.
                ctx.number += delta.parse::<i32>()?;
            }
            // Return an error from the Command which is handled by the Client.
            None => return error!("the flag 'delta' is not set"),
        }

        // No error.
        Ok(())
    })

    // Commands can also take any number of flags as input during their invocation.
    // This line define a flag delta which specifies the amount to increase number by.
    .flag("delta", Flag::Single)

    // You can specify that a Command should run in parallel with this function.
    .parallelize(true)
}

// The command d displays the current value of the number.
pub fn d() -> Command<Context> {
    Command::new(|bundle| {
        let ctx = bundle.lock();
        println!("current number: {}", ctx.number);
        println!(
            "hash size: {}\nnumber of threads: {}",
            // Get the values of options from the context.
            ctx.get_spin_option("Hash").unwrap(),
            ctx.get_spin_option("Threads").unwrap()
        );
        Ok(())
    })
}

```

## Features
- Easily build engines for **all UXI protocols**, with UGI coming out of the box.
- _**Fearless Concurrency™**_ for your Commands with a well built system for running Commands in parallel.
- Add a new option to your engine with as little as a single line of code.
- Many common Commands come inbuilt with the Client, including `setoption`, `ugi`, `isready`, and `quit`.
- **Communicate by sharing state**; custom contexts allow you to store as much persistent data as you need, all thread safe.
- Error handling inside your Commands is **as easy as using [`anyhow`](https://github.com/dtolnay/anyhow)**, all due to the powerful `RunError` type.

Refer to the [documentation](https://docs.rs/uxi) for a full in depth list of features and functions.
