use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::error::CogPoolError;
use crate::task::Task;
use crate::types::{CogType, TaskId};

pub enum CogResult<T> {
    Running,
    Cancelled,
    Paniced,
    Ok(T),
}

pub struct CogPool<T>
where
    T: CogType,
{
    id: TaskId,
    tasks: Vec<Task<T, Box<dyn FnOnce() -> T + Send>>>,
    runner: Option<JoinHandle<()>>,
    results: Arc<Mutex<HashMap<TaskId, CogResult<T>>>>,
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
            tasks: Vec::new(),
            runner: None,
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_task<F>(&mut self, func: F) -> TaskId
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let id = self.id;
        self.tasks.push(Task::new(id, Box::new(func)));
        self.id += 1;
        id
    }

    pub fn get_result(&self, id: TaskId) -> Result<T, CogPoolError> {
        if id > self.id - 1 {
            return Err(CogPoolError::TaskNotFound(id));
        }
        let results = self.results.lock().unwrap();
        match results.get(&id) {
            Some(boxed_result) => match boxed_result {
                CogResult::Ok(value) => Ok(value.clone()),
                _ => todo!(),
            },
            None => Err(CogPoolError::TaskNotFound(id)),
        }
    }

    pub fn wait_for_result(&self, id: TaskId) -> Result<T, CogPoolError> {
        loop {
            let results = self.results.lock().unwrap();
            match results.get(&id) {
                Some(boxed_result) => match boxed_result {
                    CogResult::Ok(value) => return Ok(value.clone()),
                    _ => todo!(),
                },
                None => (),
            };
        }
    }

    pub fn run(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        let results: Arc<Mutex<HashMap<TaskId, CogResult<T>>>> = Arc::clone(&self.results);
        self.runner = Some(std::thread::spawn(move || {
            for mut task in tasks {
                let id = task.id;
                let result = match task.run() {
                    Ok(result) => result,
                    Err(_) => continue,
                };
                results.lock().unwrap().insert(id, CogResult::Ok(result));
            }
        }));
    }
}
