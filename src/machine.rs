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
/// The Machine manages the engine (worker) and cogs (tasks)
/// and provides some basic methods to initialize and insert cogs,
/// as well as retrieving their results.
pub struct Machine<T>
where
    T: CogType,
{
    cog_id: CogId,
    cogs: HashMap<CogId, ArcMutexCog<T>>,
    cog_queue: Arc<Mutex<VecDeque<ArcMutexCog<T>>>>,
    engines: Vec<JoinHandle<()>>,
}

impl<T: CogType> Drop for Machine<T> {
    fn drop(&mut self) {
        // for engine in std::mem::take(&mut self.engines) {
        //     engine.join().expect("Failed to join thread");
        // }
    }
}

impl<T: CogType> Default for Machine<T> {
    fn default() -> Self {
        Self::powered(1)
    }
}

impl<T: CogType> Machine<T> {
    /// Creates a new, powered Machine
    ///
    /// Initialize a Machine without any cogs with the boilers already running
    ///
    /// # Notes
    /// - Each machine can only run cogs with the same return types.
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    ///
    /// let i32_machine = Machine::<i32>::powered(4);
    /// ```
    pub fn powered(max_engines: u32) -> Self {
        let mut machine = Self {
            cog_id: 0,
            cogs: HashMap::new(),
            cog_queue: Arc::new(Mutex::new(VecDeque::new())),
            engines: Vec::new(),
        };

        machine.spawn_engines(max_engines);
        machine
    }

    fn spawn_engines(&mut self, amount: u32) {
        for _ in 0..amount {
            let cogs = self.cog_queue.clone();
            self.engines.push(std::thread::spawn(move || {
                loop {
                    let to_run = cogs.lock().unwrap().pop_front();
                    if let Some(cog) = to_run {
                        let _ = cog.lock().unwrap().run();
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }));
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
    /// let mut machine = Machine::<i32>::default();
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

    /// Retrieves the result of a cog (task) by its ID, removing the cog once the result is
    /// retrieved.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The cog has not been added to the machine (`CogError::NotFound`).
    /// - The cog has already been retrieved (`CogError::NotFound`).
    /// - The cog has not completed (`CogError::NotCompleted`).
    /// - The cog panicked (`CogError::Panicked`).
    ///
    /// # Example
    /// NOTE: The example uses wait_for_result() to retrieve the result of the cog.
    /// This is to keep the program running synchronously
    ///
    /// ```
    /// use rustycog::Machine;
    /// use rustycog::error::CogError;
    ///
    /// let mut machine = Machine::<i32>::default();
    /// let id = machine.insert_cog(|| 42);
    ///
    /// // First retrieval - succeeds
    /// assert_eq!(machine.wait_for_result(id), Ok(42));
    ///
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(id), Err(CogError::NotInserted(id)));
    pub fn get_result(&mut self, id: CogId) -> Result<T, CogError> {
        let result = match self.cogs.get(&id) {
            Some(cog) => cog.lock().unwrap().get_result(),
            None => Err(CogError::NotInserted(id)),
        };
        match result {
            Ok(_) | Err(CogError::Panicked) => {
                self.cogs.remove(&id);
            }
            _ => (),
        }
        result
    }

    /// Waits for the result of a cog (task) by its ID, removing the cog once the result is
    /// retrieved.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The cog has not been added to the machine (`CogError::NotFound`).
    /// - The cog panicked (`CogError::Panicked`).
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    /// use rustycog::error::CogError;
    ///
    /// let mut machine = Machine::<i32>::default();
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {
    ///     panic!("I paniced :(");
    ///     0
    /// });
    ///
    /// assert_eq!(machine.wait_for_result(cog1_id), Ok(0));
    /// assert_eq!(machine.wait_for_result(cog2_id), Err(CogError::Panicked));
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(cog2_id), Err(CogError::NotInserted(cog2_id)));
    /// ```
    pub fn wait_for_result(&mut self, id: CogId) -> Result<T, CogError> {
        let cog = self.cogs.get(&id).ok_or(CogError::NotInserted(id))?;

        {
            let locked_cog = cog.lock().unwrap();
            if let CogState::Waiting = &locked_cog.state {
                let (lock, cvar) = &*locked_cog.done.clone();
                // Let the cog be run
                drop(locked_cog);

                let mut started = lock.lock().unwrap();
                while !*started {
                    started = cvar.wait(started).unwrap();
                }
            };
        }

        let result = cog.lock().unwrap().get_result();

        if matches!(result, Ok(_) | Err(CogError::Panicked)) {
            self.cogs.remove(&id);
        }
        result
    }
}
