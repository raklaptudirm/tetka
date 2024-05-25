// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::client::*;
pub use self::cmd::*;
pub use self::context::*;
pub use self::flag::*;
pub use self::parameter::*;

// Non-namespaced modules.
mod client;
mod cmd;
mod context;
mod flag;
mod parameter;

// Namespaced modules.
pub mod inbuilt;
