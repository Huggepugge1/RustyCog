use std::{
    fmt::{Debug, Formatter, Result as FormatResult},
    sync::{Arc, Condvar, Mutex},
};

use crate::{
    error::CogPoolError,
    types::{CogId, CogType},
};

pub enum CogState<T> {
    Waiting,
    Running,
    Cancelled,
    Panicked,
    Done(T),
}

pub struct Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    pub id: CogId,
    pub done: Arc<(Mutex<bool>, Condvar)>,
    pub state: CogState<T>,
    func: Option<F>,
}

impl<T, F> Debug for Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

impl<T, F> Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T,
{
    pub fn new(id: CogId, func: F) -> Self {
        Self {
            id,
            done: Arc::new((Mutex::new(false), Condvar::new())),
            func: Some(func),
            state: CogState::Waiting,
        }
    }

    pub fn get_result(&self) -> Result<T, CogPoolError> {
        match &self.state {
            CogState::Waiting => Err(CogPoolError::TaskNotCompleted),
            CogState::Done(value) => Ok(value.clone()),
            _ => todo!(),
        }
    }

    pub fn run(&mut self) -> Result<(), CogPoolError> {
        let func = std::mem::take(&mut self.func);
        if let Some(func) = func {
            println!("Running task {}", self.id);

            let result = func();
            let (lock, cvar) = &*self.done;
            let mut done = lock.lock().unwrap();
            *done = true;
            cvar.notify_one();
            self.state = CogState::Done(result);
            Ok(())
        } else {
            Err(CogPoolError::TaskAlreadyRan)
        }
    }
}
