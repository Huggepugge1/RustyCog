use std::any::type_name;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{any::Any, thread::JoinHandle};

use crate::error::CogPoolError;
use crate::task::Task;
use crate::types::TaskId;

pub struct CogPool {
    id: TaskId,
    tasks: Vec<Task<Box<dyn Any + Send>>>,
    runner: Option<JoinHandle<()>>,
    results: Arc<Mutex<HashMap<TaskId, Box<dyn Any + Send>>>>,
}

impl Drop for CogPool {
    fn drop(&mut self) {
        if let Some(runner) = self.runner.take() {
            runner.join().expect("Failed to join thread");
        }
    }
}

impl CogPool {
    pub fn new() -> Self {
        Self {
            id: 0,
            tasks: Vec::new(),
            runner: None,
            results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_task<T>(&mut self, func: T) -> TaskId
    where
        T: FnOnce() -> () + Send + 'static,
    {
        self.tasks
            .push(Task::new(self.id, move || -> Box<dyn Any + Send> {
                func();
                Box::new(())
            }));
        self.id += 1;
        self.id
    }

    pub fn add_task_with_result<T>(&mut self, func: T) -> TaskId
    where
        T: FnOnce() -> Box<dyn Any + Send> + Send + 'static,
    {
        let id = self.id;
        self.tasks.push(Task::new(id, func));
        self.id += 1;
        id
    }

    pub fn get_result<T>(&self, id: TaskId) -> Result<T, CogPoolError>
    where
        T: Clone + 'static,
    {
        if id > self.id - 1 {
            return Err(CogPoolError::TaskNotFound(id));
        }
        let results = self.results.lock().unwrap();
        match results.get(&id) {
            Some(boxed_result) => boxed_result
                .downcast_ref::<T>()
                .cloned()
                .ok_or(CogPoolError::TypeMismatch(type_name::<T>().to_string())),
            None => Err(CogPoolError::TaskNotCompleted),
        }
    }

    pub fn wait_for_result<T>(&self, id: TaskId) -> Result<T, CogPoolError>
    where
        T: Clone + 'static,
    {
        loop {
            let results = self.results.lock().unwrap();
            match results.get(&id) {
                Some(boxed_result) => {
                    return boxed_result
                        .downcast_ref::<T>()
                        .cloned()
                        .ok_or(CogPoolError::TypeMismatch(type_name::<T>().to_string()));
                }
                None => (),
            };
        }
    }

    pub fn run(&mut self) {
        let tasks = std::mem::take(&mut self.tasks);
        let results: Arc<Mutex<HashMap<TaskId, Box<dyn Any + Send>>>> = Arc::clone(&self.results);
        self.runner = Some(std::thread::spawn(move || {
            for task in tasks {
                let id = task.id;
                let result = task.run();
                results.lock().unwrap().insert(id, result);
            }
        }));
    }
}
