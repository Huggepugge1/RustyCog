use thiserror::Error;

use crate::cog::CogId;

/// Represents errors that can occur when interacting with a Cog (task).
#[derive(Error, Debug, PartialEq)]
pub enum CogError {
    /// A cog with the ID was not found in the [`Machine`](crate::Machine).
    ///
    /// This error typically occurs when trying to get the result of a Cog
    /// that was never inserted or has already been retrieved.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::{CogError, MachineError}, cog::Cog};
    ///
    /// let mut machine = Machine::<Cog<i32>>::powered(8);
    /// let non_existent_id = 999;
    ///
    /// assert_eq!(
    ///     machine.get_result(non_existent_id),
    ///     Err(MachineError::CogError(CogError::NotInserted(999)))
    /// );
    /// ```
    #[error("Cog {0} not found")]
    NotInserted(CogId),

    /// The cog has been marked as removed from it's [`Machine`](crate::Machine) but the cog
    /// was still in the machine, but the machine tried to access it.
    ///
    /// This error indicates that the cog was accessed after the cog was removed
    /// from the machine which is typically a bug in the internal logic of RustyCog.
    /// Please report this if encountered.
    #[error("Cog {0} has been removed")]
    Removed(CogId),

    /// The cog has not yet completed its execution.
    ///
    /// This error may occur when trying to get the result of a waiting or running task.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::{CogError, MachineError}, cog::Cog};
    ///
    /// let mut machine = Machine::powered(8);
    /// let cog_id = machine.insert_cog(Cog::new(|| {
    ///     std::thread::sleep(std::time::Duration::from_secs(2));
    ///     42
    /// }));
    ///
    /// assert_eq!(
    ///     machine.get_result(cog_id),
    ///     Err(MachineError::CogError(CogError::NotCompleted(cog_id)))
    /// );
    /// ```
    #[error("Cog {0} has not completed yet")]
    NotCompleted(CogId),
}

/// Represents errors that can occur when interacting with a [`Machine`](crate::Machine) (task manager).
#[derive(Error, Debug, PartialEq)]
pub enum MachineError {
    /// The machine is already powered.
    ///
    /// This usually happens when [`Machine::powered`](crate::Machine::powered)
    /// is called after [`Machine::powered`](crate::Machine::powered) has already been
    /// called.
    #[error("Machine already powered")]
    AlreadyPowered,

    /// The [`Machine`](crate::Machine) is not powered.
    ///
    /// This error indicates that someone tried to do a blocking operation
    /// on a `Machine`, but the machine was not powered
    /// This usually happens when [`Machine::powered`](crate::Machine::powered) has not been called after
    /// creating a `Machine` with [`Machine::cold`](crate::Machine::cold).
    #[error("Machine, Not powered")]
    NotPowered,

    /// A cog-related error occured.
    #[error("{0}")]
    CogError(#[from] CogError),

    /// An reciever error
    #[error("{0}")]
    RecvError(#[from] RecvError),
}

/// Errors returned by default [`Cog`](crate::cog::Cog) and [`PrioCog`](crate::cog::Cog) implementations.    
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

/// Represents errors that can occur when dealing with oneshot channels
#[derive(Error, Debug, PartialEq)]
pub enum RecvError {
    /// The oneshot value was consumed twice
    #[error("Value consumed")]
    ValueConsumed,
}
