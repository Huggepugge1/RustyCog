use thiserror::Error;

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
    /// let mut machine = Machine::<i32>::default();
    /// let non_existent_id = 999;
    ///
    /// assert_eq!(machine.get_result(non_existent_id), Err(CogError::NotInserted(999)));
    /// ```
    #[error("Cog not found with ID: {0}")]
    NotInserted(i32),

    /// The Cog (task) has been marked as removed from it's Machine but the Cog
    /// was still in the Machine and the Machine tried to access it.
    ///
    /// This error indicates that the Cog was accessed after the Cog was removed
    /// from the machine which is typically a bug in the internal logic of RustyCog.
    /// Please report this if encountered.
    #[error("Cog has not completed yet")]
    Removed,

    /// The Cog (task) has not yet completed its execution.
    ///
    /// This error may occur when trying to get the result of a waiting or running task.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError};
    ///
    /// let mut machine = Machine::<i32>::default();
    /// let cog_id = machine.insert_cog(|| {
    ///     std::thread::sleep(std::time::Duration::from_secs(2));
    ///     42
    /// });
    ///
    /// assert_eq!(machine.get_result(cog_id), Err(CogError::NotCompleted));
    /// ```
    #[error("Cog has not completed yet")]
    NotCompleted,

    /// The Cog (task) panicked during execution.
    ///
    /// This error occurs if the Cog encountered a panic while running.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError};
    ///
    /// let mut machine = Machine::<i32>::default();
    /// let cog_id = machine.insert_cog(|| panic!("Task panicked :("));
    ///
    /// assert_eq!(machine.wait_for_result(cog_id), Err(CogError::Panicked));
    /// ```
    #[error("Cog panicked")]
    Panicked,

    /// The Cog (task) has already run and cannot be run again.
    ///
    /// This error indicates that the Cog was attempted to be run multiple times,
    /// which is typically a bug in the internal logic of rustycog.
    /// Please report this if encountered.
    #[error("Cog already ran")]
    AlreadyRan,
}
