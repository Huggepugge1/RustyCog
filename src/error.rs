use thiserror::Error;

#[derive(Error, Debug)]
pub enum CogError {
    #[error("Cog not found with ID: {0}")]
    NotFound(i32),

    #[error("Cog has not completed yet")]
    NotCompleted,

    #[error("Cog was cancelled")]
    Cancelled,

    #[error("Cog panicked")]
    Panicked,

    #[error("Cog already ran")]
    AlreadyRan,
}

#[derive(Error, Debug)]
pub enum MachineError {
    #[error("CogError: {0}")]
    CogError(#[from] CogError),
}
