use std::{
    fmt::{Debug, Formatter, Result as FormatResult},
    sync::mpsc::Sender,
};

use crate::{
    error::CogError,
    types::{CogId, CogType},
};

pub struct Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    pub id: CogId,
    pub sender: Sender<Result<T, CogError>>,
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
    pub fn new(id: CogId, sender: Sender<Result<T, CogError>>, func: F) -> Self {
        Self {
            id,
            sender,
            func: Some(func),
        }
    }

    pub fn run(&mut self) -> Result<(), CogError> {
        let func = std::mem::take(&mut self.func).ok_or(CogError::AlreadyRan(self.id))?;
        match std::panic::catch_unwind(func) {
            Ok(result) => match self.sender.send(Ok(result)) {
                Ok(_) => Ok(()),
                Err(_e) => Err(CogError::SendError),
            },
            Err(_err) => Err(CogError::Panicked(self.id)),
        }
    }
}
