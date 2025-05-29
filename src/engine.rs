use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{self, AtomicBool},
        mpsc::Receiver,
    },
    thread::JoinHandle,
};

use crate::{cog::CogTrait, machine::MachineMessage};

pub type EngineId = usize;

#[derive(Debug)]
pub struct Engine {
    _id: EngineId,

    pub handle: Option<JoinHandle<()>>,

    pub ready: Arc<AtomicBool>,
    ready_engines: Arc<(Mutex<usize>, Condvar)>,
}

impl Engine {
    pub fn new<C>(
        id: usize,
        ready_engines: Arc<(Mutex<usize>, Condvar)>,
        work_receiver: Receiver<MachineMessage<C>>,
    ) -> Self
    where
        C: CogTrait,
    {
        let mut engine = Self {
            _id: id,

            handle: None,

            ready: Arc::new(AtomicBool::new(true)),
            ready_engines,
        };
        let handle = Some(engine.run(work_receiver));
        engine.handle = handle;
        engine
    }

    fn run<C, T>(&mut self, work_receiver: Receiver<MachineMessage<C>>) -> JoinHandle<()>
    where
        C: CogTrait<T = T>,
        T: Send,
    {
        let ready_engines = self.ready_engines.clone();
        let ready = self.ready.clone();
        // let id = self.id;

        std::thread::spawn(move || {
            {
                let (lock, cvar) = &*ready_engines;
                *lock.lock().unwrap() += 1;
                cvar.notify_all();
            }
            loop {
                match work_receiver.recv() {
                    Ok(value) => match value {
                        MachineMessage::Work(mut cog, sender) => {
                            ready.store(false, atomic::Ordering::Relaxed);
                            let (lock, _cvar) = &*ready_engines;
                            *lock.lock().unwrap() -= 1;

                            let result = cog.run();
                            sender.send(result);
                            ready.store(true, atomic::Ordering::Relaxed);

                            let (lock, cvar) = &*ready_engines;
                            *lock.lock().unwrap() += 1;
                            cvar.notify_all();
                        }
                        _ => return,
                    },
                    Err(_e) => {
                        return;
                    }
                }
            }
        })
    }
}
