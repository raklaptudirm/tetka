//! uxi is a package used to build UXI protocol compliant game engines easily.
//! A [Client] is the main representation of a game engine, refer to its
//! documentation for more information. The commands which engine supports being
//! sent to it from a GUI or other is represented as a [Command], refer to its
//! documentation for more details.

// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::bundles::{Bundle, BundledCtx};
pub use self::client::*;
pub use self::cmd::*;
pub use self::flag::Flag;
pub use self::parameter::Parameter;

// Non-namespaced modules.
mod bundles;
mod client;
mod cmd;
mod flag;
mod inbuilt;
mod parameter;
