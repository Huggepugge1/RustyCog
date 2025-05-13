use thiserror::Error;

#[derive(Error, Debug)]
pub enum CogPoolError {
    #[error("Task not found with ID: {0}")]
    TaskNotFound(i32),

    #[error("Type mismatch for task result. Found {0}")]
    TypeMismatch(String),

    #[error("Task has not completed yet")]
    TaskNotCompleted,
}
