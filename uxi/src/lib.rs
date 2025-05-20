//! uxi is a package used to build UXI protocol compliant game engines easily.
//!
//! A [Client] is the main representation of a game engine in uxi. It needs to
//! be initialized with information about the engine, including metadata such
//! as its name, author, and the UXI protocol it supports.
//!
//! A Client also needs to be initialized with [commands][Command], each of
//! which represent an UXI command like `uxinewgame`, `position`, `go`, etc.
//! A Client can also be initialized with a number of [parameters][Parameter].
//! A Parameter in uxi represents the schema of an UXI option.
//!
//! ```
//! # use uxi::Client;
//! # type Context = u64;
//! // Define the details of your engine in a Client.
//! let client = Client::<Context>::new()
//!     .protocol("uci")
//!     .engine("Engine v0.0.0")
//!     .author("Rak Laptudirm");
//!
//! // Run some given command with the Client.
//! client.run_cmd_string("uci");
//!
//! // Start the engine's command loop.
//! client.start();
//! ```
//!
//! Refer to the documentation for [`Client`] for more details.

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bundles::*;
pub use self::client::*;
pub use self::cmd::*;
pub use self::flag::*;
pub use self::parameter::*;

// Non-namespaced modules.
mod bundles;
mod client;
mod cmd;
mod flag;
mod inbuilt;
mod parameter;
