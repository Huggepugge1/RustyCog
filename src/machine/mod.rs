use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::cog::{Cog, CogState};
use crate::error::CogError;
use crate::types::{CogId, CogType};

type CogFn<T> = Box<dyn FnOnce() -> T + Send + std::panic::UnwindSafe + 'static>;
type ArcMutexCog<T> = Arc<Mutex<Cog<T, CogFn<T>>>>;

/// RustyCogs task manager
///
/// The Machine manages the bolier (worker) and cogs (tasks)
/// and provides some basic methods to initialize and insert cogs,
/// as well as retrieving their results.
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
    /// Creates a new Machine
    ///
    /// Initialize a Machine without any cogs
    ///
    /// # Notes
    /// - Each machine can only run cogs with the same return types.
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    /// use std::any::Any;
    ///
    /// let i32_machine = Machine::<i32>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cog_id: 0,
            cogs: HashMap::new(),
            cog_queue: Arc::new(Mutex::new(VecDeque::new())),
            boiler: None,
        }
    }

    /// Insert a cog to the machine
    ///
    /// Inserts a cog (task) to the machine.
    ///
    /// # Notes:
    /// - As of rustycog v0.1, the insert_cog does not engage (run) the cog
    ///   In the future, insert_cog will schedule the cog for engagement
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    ///
    /// let mut machine = Machine::<i32>::new();
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {1});
    /// ```
    pub fn insert_cog<F>(&mut self, func: F) -> CogId
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

    /// Get the Result of a cog
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The cog has not been added to the machine (`CogError::NotFound`).
    /// - The cog has not completed (`CogError::NotCompleted`).
    /// - The cog panicked (`CogError::Panicked`).
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    /// use rustycog::error::{CogError};
    ///
    /// let mut machine = Machine::<i32>::new();
    ///
    /// let cog_id = machine.insert_cog(|| {
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     0
    /// });
    ///
    /// machine.run();
    ///
    /// assert_eq!(machine.get_result(cog_id), Err(CogError::NotCompleted));
    ///
    /// // Ensure the cog finishes
    /// std::thread::sleep(std::time::Duration::from_secs(3));
    ///
    /// assert_eq!(machine.get_result(cog_id), Ok(0));
    /// ```
    pub fn get_result(&self, id: CogId) -> Result<T, CogError> {
        match self.cogs.get(&id) {
            Some(cog) => Ok(cog.lock().unwrap().get_result()?),
            None => Err(CogError::NotInserted(id)),
        }
    }

    /// Wait for the cog to finish, then get the result
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The cog has not been added to the machine (`CogError::NotFound`).
    /// - The cog panicked (`CogError::Panicked`).
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    /// use rustycog::error::{CogError};
    ///
    /// let mut machine = Machine::<i32>::new();
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {
    ///     panic!("I paniced :(");
    ///     0
    /// });
    ///
    /// machine.run();
    ///
    /// assert_eq!(machine.wait_for_result(cog1_id), Ok(0));
    /// assert_eq!(machine.wait_for_result(cog2_id), Err(CogError::Panicked));
    /// ```
    pub fn wait_for_result(&self, id: CogId) -> Result<T, CogError> {
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
                    _ => Ok(cog.get_result()?),
                }
            }
            None => Err(CogError::NotInserted(id)),
        }
    }

    /// Starts the boilers, power up the machine and engage (execute) all the cogs
    ///
    /// **Warning**: This is an early prototype of rustycog and this function
    /// will get replaced in the near future by cogs automatically getting
    /// scheduled and engaging
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    /// use rustycog::error::{CogError};
    ///
    /// let mut machine = Machine::<i32>::new();
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {
    ///     panic!("I paniced :(");
    ///     0
    /// });
    ///
    /// machine.run();
    /// ```
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
