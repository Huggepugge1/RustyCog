use std::fmt::{Debug, Formatter, Result as FormatResult};

use crate::{
    error::CogError,
    oneshot::Sender,
    types::{CogId, CogType},
};

pub struct Cog<T, F>
where
    T: CogType,
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    pub id: CogId,
    pub sender: Option<Sender<Result<T, CogError>>>,
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
            sender: Some(sender),
            func: Some(func),
        }
    }

    pub fn run(&mut self) -> Result<(), CogError> {
        let func = std::mem::take(&mut self.func).ok_or(CogError::AlreadyRan(self.id))?;
        let sender = std::mem::take(&mut self.sender).unwrap();
        match std::panic::catch_unwind(func) {
            Ok(result) => {
                sender.send(Ok(result));
                Ok(())
            }
            Err(_err) => {
                let err = Err(CogError::Panicked(self.id));
                sender.send(err);
                Err(CogError::Panicked(self.id))
            }
        }
    }
}
