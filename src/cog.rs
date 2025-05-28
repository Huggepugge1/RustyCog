use std::cmp::Ordering;

use crate::{error::CogError, types::CogType};

pub trait CogTrait<T>
where
    T: CogType,
{
    fn run(&mut self) -> Result<T, CogError>;
    fn priority(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T, F> From<F> for Cog<T>
where
    T: CogType,
    F: FnOnce() -> T + Send + 'static,
{
    fn from(func: F) -> Cog<T> {
        Cog::new(func)
    }
}

pub struct Cog<T>
where
    T: CogType,
{
    func: Option<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T> CogTrait<T> for Cog<T>
where
    T: CogType,
{
    fn run(&mut self) -> Result<T, CogError> {
        let func = std::mem::take(&mut self.func).ok_or(CogError::AlreadyRan)?;
        Ok(func())
    }
}

impl<T> Cog<T>
where
    T: CogType,
{
    pub fn new<F>(func: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            func: Some(Box::new(func)),
        }
    }
}

pub struct PrioCog<T>
where
    T: CogType,
{
    prio: usize,
    func: Option<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T> CogTrait<T> for PrioCog<T>
where
    T: CogType,
{
    fn run(&mut self) -> Result<T, CogError> {
        let func = std::mem::take(&mut self.func).ok_or(CogError::AlreadyRan)?;
        Ok(func())
    }

    fn priority(&self, other: &Self) -> Ordering {
        self.prio.cmp(&other.prio)
    }
}

impl<T> PrioCog<T>
where
    T: CogType,
{
    pub fn new<F>(func: F, prio: usize) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            prio,
            func: Some(Box::new(func)),
        }
    }
}
