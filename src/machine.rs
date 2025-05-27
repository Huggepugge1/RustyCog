use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::error::MachineError;
use crate::{
    cog::Cog,
    engine::Engine,
    error::CogError,
    oneshot::{Receiver, channel},
    types::{CogId, CogType, EngineId},
};

type CogFn<T> = Box<dyn FnOnce() -> T + Send + Sync + std::panic::UnwindSafe + 'static>;
type ShortCog<T> = Cog<T, CogFn<T>>;

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
    receivers: HashMap<CogId, Receiver<Result<T, CogError>>>,
    cog_results: HashMap<CogId, T>,

    max_engines: u32,
    engine_id: EngineId,
    engines: Arc<RwLock<Vec<Arc<RwLock<Engine<T>>>>>>,

    work: Arc<(Mutex<bool>, Condvar)>,
}

impl<T: CogType> Drop for Machine<T> {
    fn drop(&mut self) {
        let engines = std::mem::take(&mut self.engines);
        for engine in engines.read().unwrap().iter() {
            engine.write().unwrap().kill();
        }
    }
}

impl<T: CogType> Machine<T> {
    /// Creates a new, powered Machine
    ///
    /// Initialize a Machine without any cogs with the engines already running
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
            receivers: HashMap::new(),
            cog_results: HashMap::new(),

            engine_id: 0,
            max_engines,
            engines: Arc::new(RwLock::new(Vec::new())),

            work: Arc::new((Mutex::new(false), Condvar::new())),
        };

        machine.spawn_engines(max_engines);
        machine
    }

    /// Creates a new, cold Machine
    ///
    /// Initialize a Machine without any cogs and no engines running.
    /// To begin running cogs, Machine::power() must be called.
    ///
    /// # Notes
    /// - Each machine can only run cogs with the same return types.
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    ///
    /// let i32_machine = Machine::<i32>::cold(4);
    /// ```
    pub fn cold(max_engines: u32) -> Self {
        Self {
            cog_id: 0,
            receivers: HashMap::new(),
            cog_results: HashMap::new(),

            engine_id: 0,
            max_engines,
            engines: Arc::new(RwLock::new(Vec::new())),

            work: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// Power on a cold Machine
    ///
    /// A machine being powered means the machine can run cogs.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The machine is already powered (`MachineError::AlreadyPowered`)
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::MachineError};
    /// let mut machine = Machine::<i32>::cold(4);
    ///
    /// let powered = machine.power();
    /// assert_eq!(powered, Ok(()));
    ///
    /// let powered = machine.power();
    /// assert_eq!(powered, Err(MachineError::AlreadyPowered));
    /// ```
    pub fn power(&mut self) -> Result<(), MachineError> {
        if self.engines.read().unwrap().len() == 0 {
            self.spawn_engines(self.max_engines);
            Ok(())
        } else {
            Err(MachineError::AlreadyPowered)
        }
    }

    fn spawn_engines(&mut self, amount: u32) {
        for _ in 0..amount {
            let engines = self.engines.clone();
            self.engines.write().unwrap().push(Engine::new(
                self.engine_id,
                engines,
                self.work.clone(),
            ));
            self.engine_id += 1;
        }
    }

    /// Insert a cog into the machine
    ///
    /// Inserts a cog (task) into the machine.
    ///
    /// # Example
    /// ```
    /// use rustycog::Machine;
    ///
    /// let mut machine = Machine::powered(4);
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {1});
    /// ```
    pub fn insert_cog<F>(&mut self, func: F) -> CogId
    where
        F: FnOnce() -> T + Send + Sync + std::panic::UnwindSafe + 'static,
    {
        let id = self.cog_id;
        let (sender, receiver) = channel::<Result<T, CogError>>();
        let cog: ShortCog<T> = Cog::new(id, sender, Box::new(func));
        self.receivers.insert(id, receiver);
        self.distribute_cog(cog);

        self.cog_id += 1;
        id
    }

    pub fn insert_cog_batch<F>(&mut self, funcs: Vec<F>)
    where
        F: FnOnce() -> T + Send + Sync + std::panic::UnwindSafe + 'static,
    {
        let mut cog_batch = Vec::new();
        for func in funcs {
            let id = self.cog_id;
            let (sender, receiver) = channel::<Result<T, CogError>>();
            let cog: ShortCog<T> = Cog::new(id, sender, Box::new(func));
            self.receivers.insert(id, receiver);
            cog_batch.push(cog);
            self.cog_id += 1;
        }
        self.distribute_cog_batch(cog_batch);
    }

    fn distribute_cog(&self, cog: ShortCog<T>) {
        let cog_id = cog.id;
        if self.engines.read().unwrap().len() > 0 {
            let engine =
                self.engines.read().unwrap()[cog_id % self.engines.read().unwrap().len()].clone();
            let engine = engine.write().unwrap();
            engine.local_queue.write().unwrap().push_back(cog);

            self.notify_work();
        }
    }

    fn distribute_cog_batch(&self, cogs: Vec<ShortCog<T>>) {
        let cog_id = cogs[0].id;
        if self.engines.read().unwrap().len() > 0 {
            let engine =
                self.engines.read().unwrap()[cog_id % self.engines.read().unwrap().len()].clone();
            let engine = engine.write().unwrap();
            engine.local_queue.write().unwrap().extend(cogs);

            self.notify_work();
        }
    }

    fn notify_work(&self) {
        let (lock, cvar) = &*self.work;
        let mut work = lock.lock().unwrap();
        *work = true;
        cvar.notify_all();
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
    /// let mut machine = Machine::powered(4);
    /// let id = machine.insert_cog(|| 42);
    ///
    /// // First retrieval - succeeds
    /// assert_eq!(machine.wait_for_result(id), Ok(42));
    ///
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(id), Err(CogError::NotInserted(id)));
    pub fn get_result(&mut self, id: CogId) -> Result<T, CogError> {
        let result = match self.receivers.get(&id) {
            Some(channel) => match channel.try_recv() {
                Ok(result) => result,
                Err(_e) => Err(CogError::NotCompleted(id)),
            },
            None => Err(CogError::NotInserted(id)),
        };
        match result {
            Ok(_) | Err(CogError::Panicked(_)) => {
                self.receivers.remove(&id);
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
    /// let mut machine = Machine::powered(4);
    ///
    /// let cog1_id = machine.insert_cog(|| {0});
    /// let cog2_id = machine.insert_cog(|| {
    ///     panic!("I paniced :(");
    ///     0
    /// });
    ///
    /// assert_eq!(machine.wait_for_result(cog1_id), Ok(0));
    /// assert_eq!(machine.wait_for_result(cog2_id), Err(CogError::Panicked(cog2_id)));
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(cog2_id), Err(CogError::NotInserted(cog2_id)));
    /// ```
    pub fn wait_for_result(&mut self, id: CogId) -> Result<T, CogError> {
        match self.receivers.remove(&id) {
            Some(receiver) => receiver.recv(),
            None => match self.cog_results.remove(&id) {
                Some(result) => Ok(result),
                None => Err(CogError::NotInserted(id)),
            },
        }
    }

    /// Wait for the machine (task manager) to finish
    ///
    /// Pause execution until the machine has finished running
    /// all of its cogs (tasks)
    ///
    /// # Example
    /// ```ignore
    /// use rustycog::{Machine, error::CogError};
    /// let mut machine = Machine::powered(4);
    ///
    /// for i in 0..1000 {
    ///     machine.insert_cog(move || i);
    /// }
    ///
    /// let result = 111111;
    ///
    /// let last_id = machine.insert_cog(move || {
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     result
    /// });
    ///
    /// // Wait for all tasks
    /// assert_eq!(machine.get_result(last_id), Err(CogError::NotCompleted(last_id)));
    /// machine.wait_until_done();
    /// assert_eq!(machine.get_result(last_id), Ok(result));
    /// ```
    pub fn wait_until_done(&mut self) -> Result<(), CogError> {
        let cog_receivers = std::mem::take(&mut self.receivers);
        for (id, receiver) in cog_receivers {
            let result = receiver.recv()?;
            self.cog_results.insert(id, result);
        }
        Ok(())
    }
}
