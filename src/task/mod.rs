use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::{
    error::CogPoolError,
    types::{CogType, TaskId},
};

pub struct Task<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    pub id: TaskId,
    func: Option<F>,
}

impl<T, F> Debug for Task<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl<T, F> Task<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    pub fn new(id: TaskId, func: F) -> Self {
        Self {
            id,
            func: Some(func),
        }
    }

    pub fn run(&mut self) -> Result<T, CogPoolError> {
        let func = std::mem::take(&mut self.func);
        if let Some(func) = func {
            Ok(func())
        } else {
            Err(CogPoolError::TaskAlreadyRan)
        }
    }
}
