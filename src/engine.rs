use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread::JoinHandle,
};

use crate::{
    cog::Cog,
    types::{CogType, EngineId},
};

type CogFn<T> = Box<dyn FnOnce() -> T + Send + std::panic::UnwindSafe + 'static>;
type ArcMutexCog<T> = Arc<Mutex<Cog<T, CogFn<T>>>>;

pub struct Engine<T>
where
    T: CogType,
{
    _id: EngineId,

    pub local_queue: Arc<RwLock<VecDeque<ArcMutexCog<T>>>>,

    engines: Arc<RwLock<Vec<Arc<RwLock<Engine<T>>>>>>,

    handle: Option<JoinHandle<()>>,
    termination_flag: Arc<RwLock<bool>>,

    work: Arc<(Mutex<bool>, Condvar)>,
}

impl<T> Engine<T>
where
    T: CogType,
{
    pub fn new(
        id: usize,
        engines: Arc<RwLock<Vec<Arc<RwLock<Engine<T>>>>>>,
        work: Arc<(Mutex<bool>, Condvar)>,
    ) -> Arc<RwLock<Self>> {
        let engine = Arc::new(RwLock::new(Self {
            _id: id,

            local_queue: Arc::new(RwLock::new(VecDeque::new())),

            engines,

            handle: None,
            termination_flag: Arc::new(RwLock::new(false)),

            work,
        }));
        let handle = Some(engine.read().unwrap().run(engine.clone()));
        engine.write().unwrap().handle = handle;
        engine
    }

    fn run(&self, arc_pointer: Arc<RwLock<Self>>) -> JoinHandle<()> {
        let local_queue = self.local_queue.clone();
        let termination_flag = self.termination_flag.clone();
        let engines = self.engines.clone();
        // let id = self._id;
        let work = self.work.clone();

        std::thread::spawn(move || {
            loop {
                if *termination_flag.read().unwrap() {
                    return;
                }
                if let Some(cog) = local_queue.write().unwrap().pop_front() {
                    let _ = cog.lock().unwrap().run();
                } else if let Some(cogs) = Self::cog_steal(&engines, &arc_pointer) {
                    local_queue.write().unwrap().extend(cogs);
                } else {
                    let (lock, cvar) = &*work;
                    let mut ready = lock.lock().unwrap();
                    while !*ready && !*termination_flag.read().unwrap() {
                        ready = cvar.wait(ready).unwrap();
                    }
                    *ready = false;
                }
            }
        })
    }

    fn cog_steal(
        engines: &Arc<RwLock<Vec<Arc<RwLock<Engine<T>>>>>>,
        self_pointer: &Arc<RwLock<Self>>,
    ) -> Option<VecDeque<ArcMutexCog<T>>> {
        for engine in engines.read().unwrap().iter() {
            if Arc::ptr_eq(engine, self_pointer) {
                continue;
            }
            let engine = engine.read().unwrap();
            let mut queue = engine.local_queue.write().unwrap();
            let len = queue.len();
            if len > 0 {
                return Some(
                    queue
                        .drain(0..usize::max(1, len / engines.read().unwrap().len()))
                        .collect(),
                );
            }
        }
        None
    }

    pub fn kill(&mut self) {
        *self.termination_flag.write().unwrap() = true;
        if let Some(handle) = std::mem::take(&mut self.handle) {
            self.notify_work_to_kill();
            let _ = handle.join();
        }
    }

    fn notify_work_to_kill(&self) {
        let (lock, cvar) = &*self.work;
        let mut work = lock.lock().unwrap();
        *work = true;
        cvar.notify_all();
    }
}
