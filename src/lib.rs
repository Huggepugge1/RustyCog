//! # RustyCog
//!
//! RustyCog is a high-performance, flexible task management library for Rust.
//! It allows you to create and manage "cogs" (tasks) in a "machine" (task pool).
//!
//! ## Features
//! - Type safe task execution
//! - Automatic scheduling and execution of tasks
//! - Retrieve task results with `get_result` or `wait_for_result`
//!
//! ## Quick Start
//! ```
//! use rustycog::{machine, error::CogError, cog::Cog};
//!
//! let mut machine = machine!(Cog, i32, 4);
//! let cog_id = machine.insert_cog(|| {
//!     println!("Hello, RustyCog!");
//!     42
//! });
//!
//! let result = machine.wait_for_result(cog_id).unwrap();
//! println!("Result: {:?}", result);
//! ```
//!
//! ## Dynamic Typing
//! RustyCog can also handle dynamically typed tasks, but you (the user) are responsible
//! for managing type safety if using the `Any` trait.
//! This gives you flexibility without sacrificing performance.
//!
//! ### Example 1: Using Enums (Recommended)
//! ```
//! use rustycog::{machine, error::CogError, cog::Cog};
//!
//! enum MyTypes {
//!     Int(i32),
//!     Bool(bool),
//! }
//!
//!
//! let mut machine = machine!(Cog, MyTypes, 4);
//! machine.insert_cog(|| MyTypes::Int(42));
//! machine.insert_cog(|| MyTypes::Bool(true));
//! ```
//!
//! ### Example 2: Using `Box<dyn Any>` (Advanced)
//! NOTE: You could replace `Box` with any other smart pointer, as long as it implements Send
//!
//! ```
//! use rustycog::{machine, error::CogError, cog::Cog};
//! use std::any::Any;
//!
//! let mut any_machine = machine!(Cog, Box<dyn Any + Send>, 4);
//! let id = any_machine.insert_cog(|| Box::new(42) as Box<dyn Any + Send>);
//!
//! let result = any_machine.wait_for_result(id).unwrap();
//!
//! if let Some(value) = result.downcast_ref::<i32>() {
//!     println!("Got an i32: {}", value);
//! } else {
//!     println!("Unknown type");
//! }
//! ```
//!
//! ## Error Handling
//! RustyCog provides error handling through MachineError and `CogError`.

pub mod cog;
mod dispatcher;
mod engine;
pub mod error;
mod machine;
mod oneshot;
pub mod types;

#[macro_export]
macro_rules! machine {
    ($cog: ident, $t:ty, $threads:expr) => {
        $crate::Machine::<$cog<$t>, $t>::powered($threads)
    };
}

#[macro_export]
macro_rules! cold_machine {
    ($cog: ident, $t:ty, $threads:expr) => {
        $crate::Machine::<$cog<$t>, $t>::cold($threads)
    };
}

#[doc(inline)]
pub use crate::machine::Machine;
