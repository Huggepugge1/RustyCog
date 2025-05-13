use crate::task::{Id, Task};

#[derive(Default)]
pub struct CogPool {
    id: Id,
    tasks: Vec<Task>,
}

impl Drop for CogPool {
    fn drop(&mut self) {
        todo!()
    }
}

impl CogPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task<T: Fn() + Send + 'static>(&mut self, func: T) {
        self.tasks.push(Task::new(self.id, func));
        self.id += 1;
    }

    pub fn run(&mut self) {
        for task in &self.tasks {
            task.run();
        }

        println!("{:#?}", self.tasks);
    }
}
