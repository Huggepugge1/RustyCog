use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::cog::{Cog, CogState};
use crate::error::{CogError, MachineError};
use crate::types::{CogId, CogType};

pub enum CogResult<T> {
    Running,
    Cancelled,
    Paniced,
    Ok(T),
}

type CogFn<T> = Box<dyn FnOnce() -> T + Send + std::panic::UnwindSafe + 'static>;
type ArcMutexCog<T> = Arc<Mutex<Cog<T, CogFn<T>>>>;

pub struct Machine<T>
where
    T: CogType,
{
    cog_id: CogId,
    cogs: HashMap<CogId, ArcMutexCog<T>>,
    cog_queue: Arc<Mutex<VecDeque<ArcMutexCog<T>>>>,
    boiler: Option<JoinHandle<()>>,
}

impl<T: CogType> Drop for Machine<T> {
    fn drop(&mut self) {
        if let Some(runner) = self.boiler.take() {
            runner.join().expect("Failed to join thread");
        }
    }
}

impl<T: CogType> Machine<T> {
    pub fn new() -> Self {
        Self {
            cog_id: 0,
            cogs: HashMap::new(),
            cog_queue: Arc::new(Mutex::new(VecDeque::new())),
            boiler: None,
        }
    }

    pub fn add_cog<F>(&mut self, func: F) -> CogId
    where
        F: FnOnce() -> T + Send + std::panic::UnwindSafe + 'static,
    {
        let id = self.cog_id;
        let cog: ArcMutexCog<T> = Arc::new(Mutex::new(Cog::new(id, Box::new(func))));
        self.cogs.insert(id, cog.clone());
        self.cog_queue.lock().unwrap().push_back(cog);

        self.cog_id += 1;
        id
    }

    pub fn get_result(&self, id: CogId) -> Result<T, MachineError> {
        match self.cogs.get(&id) {
            Some(cog) => Ok(cog.lock().unwrap().get_result()?),
            None => Err(MachineError::CogError(CogError::NotFound(id))),
        }
    }

    pub fn wait_for_result(&self, id: CogId) -> Result<T, MachineError> {
        match self.cogs.get(&id) {
            Some(cog) => {
                let cog_clone = cog.clone();
                let cog = cog.lock().unwrap();
                match &cog.state {
                    CogState::Waiting => {
                        let (lock, cvar) = &*cog.done.clone();
                        drop(cog);
                        let mut started = lock.lock().unwrap();
                        while !*started {
                            started = cvar.wait(started).unwrap();
                        }
                        Ok(cog_clone.lock().unwrap().get_result()?)
                    }
                    CogState::Done(value) => Ok(value.clone()),
                    _ => todo!(),
                }
            }
            None => Err(MachineError::CogError(CogError::NotFound(id))),
        }
    }

    pub fn run(&mut self) {
        let cogs = self.cog_queue.clone();
        self.boiler = Some(std::thread::spawn(move || {
            loop {
                let mut cogs = cogs.lock().unwrap();
                let to_run = cogs.pop_front();
                drop(cogs);
                if let Some(cog) = to_run {
                    let _result = match cog.lock().unwrap().run() {
                        Ok(()) => (),
                        Err(_) => (),
                    };
                } else {
                    return;
                }
            }
        }));
    }
}
