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
//! use rustycog::Machine;
//!
//! let mut machine = Machine::powered(4);
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
//! use rustycog::Machine;
//!
//! enum MyTypes {
//!     Int(i32),
//!     Bool(bool),
//! }
//!
//! let mut machine = Machine::<MyTypes>::powered(4);
//! machine.insert_cog(|| MyTypes::Int(42));
//! machine.insert_cog(|| MyTypes::Bool(true));
//! ```
//!
//! ### Example 2: Using `Box<dyn Any>` (Advanced)
//! NOTE: You could replace `Box` with any other smart pointer, as long as it implements Send
//!
//! ```
//! use rustycog::Machine;
//! use std::any::Any;
//!
//! let mut any_machine = Machine::<Box<dyn Any + Send>>::powered(4);
//! let id = any_machine.insert_cog(|| Box::new(42));
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

mod cog;
mod engine;
pub mod error;
mod machine;
pub mod types;

#[doc(inline)]
pub use crate::machine::Machine;
