use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex, mpsc::Sender},
};

use crate::{
    cog::{CogId, CogTrait},
    dispatcher::Dispatcher,
    engine::{Engine, EngineId},
    error::{CogError, MachineError},
};

pub enum MachineMessage<C>
where
    C: CogTrait,
{
    Work(C, crate::oneshot::Sender<C::T>),
    Terminate,
}

/// RustyCogs task manager
///
/// The Machine manages the engine (worker) and cogs (tasks)
/// and provides some basic methods to initialize and insert cogs,
/// as well as retrieving their results.
pub struct Machine<C>
where
    C: CogTrait,
{
    cog_id: CogId,
    receivers: HashMap<CogId, crate::oneshot::Receiver<C::T>>,
    cog_results: HashMap<CogId, C::T>,

    max_engines: u32,
    engine_id: EngineId,
    ready_engines: Arc<(Mutex<usize>, Condvar)>,

    work_sender: Option<Sender<MachineMessage<C>>>,

    powered: bool,

    queue: Option<VecDeque<(C, crate::oneshot::Sender<C::T>)>>,
}

impl<C> Drop for Machine<C>
where
    C: CogTrait,
{
    fn drop(&mut self) {
        match &self.work_sender {
            Some(sender) => {
                let _ = sender.send(MachineMessage::Terminate);
            }
            None => (),
        }
    }
}

impl<C> Machine<C>
where
    C: CogTrait,
{
    /// Creates a new, powered Machine
    ///
    /// Initialize a Machine without any cogs with the engines already running
    ///
    /// # Notes
    /// - Each machine can only run cogs with the same return type.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, cog::Cog};
    ///
    /// let machine = Machine::<Cog<i32>>::powered(8);
    /// ```
    pub fn powered(max_engines: u32) -> Self {
        let mut machine = Machine::cold(max_engines);
        let _ = machine.power();
        machine
    }

    /// Creates a new, cold Machine
    ///
    /// Initialize a Machine without any cogs and no engines running.
    /// To begin running cogs, Machine::power() must be called.
    ///
    /// # Notes
    /// - Each machine can only run cogs with the same return type.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, cog::Cog};
    ///
    /// let machine = Machine::<Cog<i32>>::cold(8);
    /// ```
    pub fn cold(max_engines: u32) -> Self {
        Self {
            cog_id: 0,
            receivers: HashMap::new(),
            cog_results: HashMap::new(),

            engine_id: 0,
            max_engines,

            ready_engines: Arc::new((Mutex::new(0), Condvar::new())),

            work_sender: None,

            powered: false,

            queue: Some(VecDeque::new()),
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
    /// use rustycog::{Machine, error::{CogError, MachineError}, cog::Cog};
    /// let mut machine = Machine::<Cog<i32>>::cold(8);
    ///
    /// let powered = machine.power();
    /// assert_eq!(powered, Ok(()));
    ///
    /// let powered = machine.power();
    /// assert_eq!(powered, Err(MachineError::AlreadyPowered));
    /// ```
    pub fn power(&mut self) -> Result<(), MachineError> {
        if !self.powered {
            self.spawn_dispatcher();
            self.powered = true;
            Ok(())
        } else {
            Err(MachineError::AlreadyPowered)
        }
    }

    fn spawn_engines(
        &mut self,
        amount: u32,
    ) -> Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<C>>)> {
        let mut engines = Vec::new();
        for _ in 0..amount {
            let (sender, receiver) = std::sync::mpsc::channel();
            engines.push((
                Engine::new(self.engine_id, self.ready_engines.clone(), receiver),
                sender,
            ));
            self.engine_id += 1;
        }
        engines
    }

    fn spawn_dispatcher(&mut self) {
        let engines = self.spawn_engines(self.max_engines);
        let (work_sender, receiver) = std::sync::mpsc::channel();
        let mut dispatcher = Dispatcher::new(engines, receiver, self.ready_engines.clone());
        std::thread::spawn(move || dispatcher.run());
        let queue = std::mem::take(&mut self.queue).unwrap();
        for (cog, sender) in queue {
            let _ = work_sender.send(MachineMessage::Work(cog, sender));
        }
        self.work_sender = Some(work_sender);
    }

    /// Insert a cog into the machine
    ///
    /// Inserts a cog (task) into the machine.
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    ///
    /// let mut machine = Machine::powered(8);
    ///
    /// let cog1_id = machine.insert_cog(Cog::new(|| 0));
    /// let cog2_id = machine.insert_cog(Cog::new(|| 1));
    /// ```
    pub fn insert_cog(&mut self, cog: C) -> CogId {
        let id = self.cog_id;
        let (sender, receiver) = crate::oneshot::channel::<C::T>();
        if let Some(work_sender) = &self.work_sender {
            let _ = work_sender.send(MachineMessage::Work(cog, sender));
        } else if let Some(ref mut queue) = self.queue {
            queue.push_back((cog.into(), sender));
        }
        self.receivers.insert(id, receiver);

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
    ///
    /// # Example
    /// NOTE: The example uses wait_for_result() to retrieve the result of the cog.
    /// This is to keep the program running synchronously
    ///
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    ///
    /// let mut machine = Machine::powered(8);
    /// let id = machine.insert_cog(Cog::new(|| 42));
    ///
    /// // First retrieval - succeeds
    /// assert_eq!(machine.wait_for_result(id).unwrap(), Ok(42));
    ///
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(id), Err(CogError::NotInserted(id)));
    pub fn get_result(&mut self, id: CogId) -> Result<C::T, CogError> {
        let result = match self.receivers.get(&id) {
            Some(channel) => match channel.try_recv() {
                Some(result) => Ok(result),
                None => Err(CogError::NotCompleted(id)),
            },
            None => match self.cog_results.remove(&id) {
                Some(result) => Ok(result),
                None => Err(CogError::NotInserted(id)),
            },
        };
        if let Ok(_) = result {
            self.receivers.remove(&id);
        }

        result
    }

    /// Waits for the result of a cog (task) by its ID, removing the cog once the result is
    /// retrieved.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The cog has not been added to the machine ([`MachineError::CogError(CogError::NotFound)`]).
    /// - The machine has not been powered ([`MachineError::NotPowered`]).
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    ///
    /// let mut machine = Machine::powered(8);
    ///
    /// let cog_id = machine.insert_cog(Cog::new(|| 0));
    ///
    /// assert_eq!(machine.wait_for_result(cog_id).unwrap(), Ok(0));
    /// // Second retrieval - cog is already removed
    /// assert_eq!(machine.wait_for_result(cog_id), Err(CogError::NotInserted(cog_id)));
    /// ```
    pub fn wait_for_result(&mut self, id: CogId) -> Result<C::T, MachineError> {
        if !self.powered {
            return Err(MachineError::NotPowered);
        }
        match self.receivers.remove(&id) {
            Some(receiver) => Ok(receiver.recv()),
            None => match self.cog_results.remove(&id) {
                Some(result) => Ok(result),
                None => Err(MachineError::CogError(CogError::NotInserted(id))),
            },
        }
    }

    /// Wait for the machine (task manager) to finish
    ///
    /// Pause execution until the machine has finished running
    /// all of its cogs (tasks)
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The machine has not been powered ([`MachineError::NotPowered`]).
    ///
    /// # Example
    /// ```
    /// use rustycog::{Machine, error::CogError, cog::Cog};
    /// let mut machine = Machine::powered(8);
    ///
    /// for i in 0..1000 {
    ///     machine.insert_cog(Cog::new(move || i));
    /// }
    ///
    /// let result = 111111;
    ///
    /// let last_id = machine.insert_cog(Cog::new(move || {
    ///     std::thread::sleep(std::time::Duration::from_secs(1));
    ///     result
    /// }));
    ///
    /// // Wait for all tasks
    /// machine.wait_until_done();
    /// assert_eq!(machine.get_result(last_id).unwrap(), Ok(result));
    /// ```
    pub fn wait_until_done(&mut self) -> Result<(), MachineError> {
        if !self.powered {
            return Err(MachineError::NotPowered);
        }
        let cog_receivers = std::mem::take(&mut self.receivers);
        for (id, receiver) in cog_receivers {
            let result = receiver.recv();
            self.cog_results.insert(id, result);
        }
        Ok(())
    }
}
