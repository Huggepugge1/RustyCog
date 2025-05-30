//! # RustyCog
//!
//! RustyCog is a high-performance, flexible task management library for Rust.
//! It allows you to create and manage cogs (tasks) in a machine (task pool).
//!
//! ## Features
//! - Type safe cog execution.
//! - Automatic scheduling and execution of cogs.
//! - Retrieve cog results with [`Machine::get_result`] or [`Machine::wait_for_result`].
//!
//! ## Quick Start
//! ```
//! use rustycog::{Machine, error::CogError, cog::Cog};
//!
//! let mut machine = Machine::powered(8);
//! let cog_id = machine.insert_cog(Cog::new(|| {
//!     println!("Hello, RustyCog!");
//!     42
//! }));
//!
//! let result = machine.wait_for_result(cog_id).unwrap();
//! assert_eq!(result, Ok(42));
//! ```
//!
//! ## Dynamic Typing
//! RustyCog can also handle dynamically typed cog, but you (the user) are responsible
//! for managing type safety if using the [`trait@std::any::Any`] trait.
//! This gives you flexibility without sacrificing performance.
//!
//! ### Example 1: Using Enums (Recommended)
//! ```
//! use rustycog::{Machine, error::CogError, cog::Cog};
//!
//! #[derive(Debug, PartialEq)]
//! enum MyTypes {
//!     Int(i32),
//!     Bool(bool),
//! }
//!
//! let mut machine = Machine::powered(8);
//! let id1 = machine.insert_cog(Cog::new(|| MyTypes::Int(42)));
//! let id2 = machine.insert_cog(Cog::new(|| MyTypes::Bool(true)));
//!
//! assert_eq!(machine.wait_for_result(id1).unwrap(), Ok(MyTypes::Int(42)));
//! assert_eq!(machine.wait_for_result(id2).unwrap(), Ok(MyTypes::Bool(true)));
//! ```
//!
//! ### Example 2: Using [`Box<dyn Any>`] (Advanced)
//! NOTE: You could replace [`Box`] with any other smart pointer, as long as it implements [`trait@Send`]
//!
//! ```
//! use rustycog::{Machine, error::CogError, cog::Cog};
//! use std::any::Any;
//!
//! let mut any_machine = Machine::powered(8);
//! let id = any_machine.insert_cog(Cog::new(|| Box::new(42) as Box<dyn Any + Send>));
//!
//! let result = any_machine.wait_for_result(id).unwrap().unwrap();
//!
//! if let Some(value) = result.downcast_ref::<i32>() {
//!     assert_eq!(*value, 42);
//! } else {
//!     assert!(false, "Expected an i32, found something else!");
//! }
//! ```
//!
//! ## Error Handling
//! RustyCog provides error handling through [`MachineError`](crate::error::MachineError) and [`CogError`](crate::error::CogError).

pub mod cog;
mod dispatcher;
mod engine;
pub mod error;
mod machine;
mod oneshot;

#[doc(inline)]
pub use crate::machine::Machine;
