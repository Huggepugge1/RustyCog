use thiserror::Error;

use crate::cog::CogId;

/// Represents errors that can occur when interacting with a Cog (task).
#[derive(Error, Debug, PartialEq)]
pub enum CogError {
    /// The specified Cog (task) ID was not found in the Machine.
    ///
    /// This error typically occurs when trying to get the result of a Cog
    /// that was never inserted or has already been removed.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    ///
    /// let mut machine = Machine::<Cog<i32>>::powered(8);
    /// let non_existent_id = 999;
    ///
    /// assert_eq!(machine.get_result(non_existent_id), Err(CogError::NotInserted(999)));
    /// ```
    #[error("Cog {0} not found")]
    NotInserted(CogId),

    /// The Cog (task) has been marked as removed from it's Machine but the Cog
    /// was still in the Machine and the Machine tried to access it.
    ///
    /// This error indicates that the Cog was accessed after the Cog was removed
    /// from the machine which is typically a bug in the internal logic of RustyCog.
    /// Please report this if encountered.
    #[error("Cog {0} has been removed")]
    Removed(CogId),

    /// The Cog (task) has not yet completed its execution.
    ///
    /// This error may occur when trying to get the result of a waiting or running task.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    ///
    /// let mut machine = Machine::powered(8);
    /// let cog_id = machine.insert_cog(Cog::new(|| {
    ///     std::thread::sleep(std::time::Duration::from_secs(2));
    ///     42
    /// }));
    ///
    /// assert_eq!(machine.get_result(cog_id), Err(CogError::NotCompleted(cog_id)));
    /// ```
    #[error("Cog {0} has not completed yet")]
    NotCompleted(CogId),
}

/// Represents errors that can occur when interacting with a `Machine` (task manager).
#[derive(Error, Debug, PartialEq)]
pub enum MachineError {
    /// The machine is already powered.
    ///
    /// This usually happens when `Machine::power()` is called after `Machine::powered()` has been
    /// called.
    #[error("Machine already powered")]
    AlreadyPowered,

    /// The Machine (task manager) is not powered.
    ///
    /// This error indicates that someone tried to do a blocking operation
    /// on a `Machine`, but the machine was not powered
    /// This usually happens when `Machine::power()` has not been called after
    /// creating a `Machine` with `Machine::cold()`.
    #[error("Machine, Not powered")]
    NotPowered,

    /// A cog-related error occured.
    #[error("{0}")]
    CogError(#[from] CogError),
}

/// Errors returned by default `Cog` and `PrioCog` implementations.
///
/// The following errors should **never happen** in normal operation.
/// If you see an error from here, it likely means **RustyCog is cooked** —  
/// there’s a logic bug in task scheduling or execution.
///
/// # Variants
/// - [`DefaultCogError::AlreadyRan`]: The cog was attempted to be run multiple times, indicating a serious internal error.
#[derive(Error, Debug, PartialEq)]
pub enum DefaultCogError {
    /// Indicates a critical logic error: cog was executed more than once.
    #[error("Cog already ran!")]
    AlreadyRan,
}
