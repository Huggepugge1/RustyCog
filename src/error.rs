use thiserror::Error;

use crate::types::CogId;

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
    /// use rustycog::{Machine, error::CogError};
    ///
    /// let mut machine = Machine::<i32>::powered(1);
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
    /// use rustycog::{Machine, error::CogError};
    ///
    /// let mut machine = Machine::powered(1);
    /// let cog_id = machine.insert_cog(|| {
    ///     std::thread::sleep(std::time::Duration::from_secs(2));
    ///     42
    /// });
    ///
    /// assert_eq!(machine.get_result(cog_id), Err(CogError::NotCompleted(cog_id)));
    /// ```
    #[error("Cog {0} has not completed yet")]
    NotCompleted(CogId),

    /// The Cog (task) panicked during execution.
    ///
    /// This error occurs if the Cog encountered a panic while running.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError};
    ///
    /// let mut machine = Machine::powered(1);
    /// let cog_id = machine.insert_cog(|| panic!("Task panicked :("));
    ///
    /// assert_eq!(machine.wait_for_result(cog_id), Err(CogError::Panicked(cog_id)));
    /// ```
    #[error("Cog {0} panicked")]
    Panicked(CogId),

    /// The Cog (task) has already run and cannot be run again.
    ///
    /// This error indicates that the Cog was attempted to be run multiple times,
    /// which is typically a bug in the internal logic of rustycog.
    /// Please report this if encountered.
    #[error("Cog {0} already ran")]
    AlreadyRan(CogId),
}

/// Represents errors that can occur when interacting with a Machine (task manager).
#[derive(Error, Debug, PartialEq)]
pub enum MachineError {
    /// The Machine (task manager) is already powered
    ///
    /// This error indicates that the machine tried to power on when it was already powered.
    /// This usually happens when Machine::power() is called after Machine::powered() has been
    /// called.
    #[error("Machine already powered")]
    AlreadyPowered,
}
