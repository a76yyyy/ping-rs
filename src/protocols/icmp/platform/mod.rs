//! Deprecated module. Use `crate::protocols::icmp::{execute_ping, execute_ping_async}` instead.

#[allow(unused_imports)]
#[deprecated(since = "2.1.0", note = "Use `crate::protocols::icmp::execute_ping` instead")]
pub use crate::protocols::icmp::execute_ping;

#[allow(unused_imports)]
#[deprecated(since = "2.1.0", note = "Use `crate::protocols::icmp::execute_ping_async` instead")]
pub use crate::protocols::icmp::execute_ping_async;
