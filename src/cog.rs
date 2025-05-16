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
    Removed,
    Done(T),
}

pub struct Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    pub id: CogId,
    pub done: Arc<(Mutex<bool>, Condvar)>,
    pub state: CogState<T>,
    func: Option<F>,
}

impl<T, F> Debug for Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T + std::panic::UnwindSafe,
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

    pub fn get_result(&mut self) -> Result<T, CogError> {
        match self.state {
            CogState::Done(_) | CogState::Panicked => {
                // Replace needs to happen since we want to move the result from Done
                // This way, in a Machine<T>, T does not have to implement Clone or Copy
                match std::mem::replace(&mut self.state, CogState::Removed) {
                    CogState::Done(result) => Ok(result),
                    CogState::Panicked => Err(CogError::Panicked),
                    _ => unreachable!(),
                }
            }

            CogState::Removed => Err(CogError::Removed),
            CogState::Waiting | CogState::Running => Err(CogError::NotCompleted),
        }
    }

    pub fn run(&mut self) -> Result<(), CogError> {
        self.state = CogState::Running;

        let func = std::mem::take(&mut self.func).ok_or(CogError::AlreadyRan)?;
        let result = match std::panic::catch_unwind(func) {
            Ok(result) => {
                self.state = CogState::Done(result);
                Ok(())
            }
            Err(_err) => {
                self.state = CogState::Panicked;
                Err(CogError::Panicked)
            }
        };

        self.notify_done();
        result
    }

    fn notify_done(&mut self) {
        let (lock, cvar) = &*self.done;
        let mut done = lock.lock().unwrap();
        *done = true;
        cvar.notify_one();
    }
}
