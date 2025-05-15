use std::{
    fmt::{Debug, Formatter, Result as FormatResult},
    sync::{Arc, Condvar, Mutex},
};

use crate::{
    error::CogError,
    types::{CogId, CogType},
};

pub enum CogState<T> {
    Waiting,
    Running,
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
        f.debug_struct("Cog").field("id", &self.id).finish()
    }
}

impl<T, F> Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    pub fn new(id: CogId, func: F) -> Self {
        Self {
            id,
            done: Arc::new((Mutex::new(false), Condvar::new())),
            func: Some(func),
            state: CogState::Waiting,
        }
    }

    pub fn get_result(&self) -> Result<T, CogError> {
        match &self.state {
            CogState::Waiting | CogState::Running => Err(CogError::NotCompleted),
            CogState::Done(value) => Ok(value.clone()),

            CogState::Panicked => Err(CogError::Panicked),
        }
    }

    pub fn run(&mut self) -> Result<(), CogError> {
        self.state = CogState::Running;
        let func = std::mem::take(&mut self.func);
        if let Some(func) = func {
            println!("Running cog {}", self.id);

            let result = match std::panic::catch_unwind(func) {
                Ok(result) => result,
                Err(_err) => {
                    self.state = CogState::Panicked;
                    let (lock, cvar) = &*self.done;
                    let mut done = lock.lock().unwrap();
                    *done = true;
                    cvar.notify_one();
                    return Err(CogError::Panicked);
                }
            };
            let (lock, cvar) = &*self.done;
            let mut done = lock.lock().unwrap();
            *done = true;
            cvar.notify_one();
            self.state = CogState::Done(result);
            Ok(())
        } else {
            Err(CogError::AlreadyRan)
        }
    }
}
