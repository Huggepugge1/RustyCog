use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::types::TaskId;

pub struct Task<T: Send> {
    pub id: TaskId,
    func: Box<dyn FnOnce() -> T + Send + 'static>,
}

impl<T: Send> Debug for Task<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl<T: Send> Task<T> {
    pub fn new<F>(id: TaskId, func: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            id,
            func: Box::new(func),
        }
    }

    pub fn run(self) -> T {
        (self.func)()
    }
}
