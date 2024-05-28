// Make the contents of the non-namespaced
// modules public, so they can be accessed
// without their parent namespace.
pub use self::client::*;
pub use self::cmd::*;
pub use self::context::{Bundle, GuardedBundledCtx};
pub use self::flag::Flag;
pub use self::inbuilt::BundledCtx;
pub use self::parameter::{Number, Parameter};

// Non-namespaced modules.
mod client;
mod cmd;
mod context;
mod flag;
mod inbuilt;
mod parameter;
