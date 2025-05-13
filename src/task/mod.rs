use std::fmt::{Debug, Formatter, Result as FormatResult};

pub type Id = i32;

pub struct Task {
    id: Id,
    func: Box<dyn Fn() + Send + 'static>,
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl Task {
    pub fn new<T: Fn() + Send + 'static>(id: Id, func: T) -> Self {
        Self {
            id,
            func: Box::new(func),
        }
    }

    pub fn run(&self) {
        (self.func)();
    }
}
