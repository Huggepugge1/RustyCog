//! # RustyCog
//!
//! RustyCog is a high-performance, flexible task management library for Rust.
//! It allows you to create and manage "cogs" (tasks) in a "machine" (task pool).
//!
//! ## Features
//! - type-safe task execution
//! - Automatic scheduling and execution of tasks
//! - Retrieve task results with `get_result` or `wait_for_result`
//!
//! ## Quick Start
//! ```rust
//! use rustycog::Machine;
//!
//! fn main() {
//!     let mut machine = Machine::<i32>::new();
//!     let cog_id = machine.insert_cog(|| {
//!         println!("Hello, RustyCog!");
//!         42
//!     });
//!     
//!     machine.run();
//!     let result = machine.wait_for_result(cog_id).unwrap();
//!     println!("Result: {:?}", result);
//! }
//! ```
//!
//! ## Error Handling
//! RustyCog provides error handling through `CogError`.

mod cog;
pub mod error;
mod machine;
mod types;

#[doc(inline)]
pub use crate::machine::Machine;
