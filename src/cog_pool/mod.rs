use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::error::CogPoolError;
use crate::task::{Cog, CogState};
use crate::types::{CogId, CogType};

pub struct CogPool<T>
where
    T: CogType,
{
    id: CogId,
    tasks: Arc<Mutex<HashMap<CogId, Arc<Mutex<Cog<T, Box<dyn FnOnce() -> T + Send>>>>>>>,
    runner: Option<JoinHandle<()>>,
}

impl<T: CogType> Drop for CogPool<T> {
    fn drop(&mut self) {
        if let Some(runner) = self.runner.take() {
            runner.join().expect("Failed to join thread");
        }
    }
}

impl<T: CogType> CogPool<T> {
    pub fn new() -> Self {
        Self {
            id: 0,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            runner: None,
        }
    }

    pub fn add_task<F>(&mut self, func: F) -> CogId
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let id = self.id;
        self.tasks
            .lock()
            .unwrap()
            .insert(id, Arc::new(Mutex::new(Cog::new(id, Box::new(func)))));
        self.id += 1;
        id
    }

    pub fn get_result(&self, id: CogId) -> Result<T, CogPoolError> {
        match self.tasks.lock().unwrap().get(&id) {
            Some(task) => match &task.lock().unwrap().state {
                CogState::Waiting => Err(CogPoolError::TaskNotCompleted),
                CogState::Done(value) => Ok(value.clone()),
                _ => todo!(),
            },
            None => todo!(),
        }
    }

    pub fn wait_for_result(&self, id: CogId) -> Result<T, CogPoolError> {
        loop {
            match self.tasks.lock().unwrap().get(&id) {
                Some(task) => match &task.lock().unwrap().state {
                    CogState::Waiting => continue,
                    CogState::Done(value) => return Ok(value.clone()),
                    _ => todo!(),
                },
                None => todo!(),
            }
        }
    }

    pub fn run(&mut self) {
        let tasks = self.tasks.clone();
        self.runner = Some(std::thread::spawn(move || {
            loop {
                let mut to_run = None;
                let mut tasks = tasks.lock().unwrap();
                for (_task_id, task) in tasks.iter_mut() {
                    let task_clone = task.clone();
                    let task = task.lock().unwrap();
                    if let CogState::Waiting = task.state {
                        to_run = Some(task_clone);
                        break;
                    }
                }
                drop(tasks);
                if let Some(task) = to_run {
                    let _result = match task.lock().unwrap().run() {
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
