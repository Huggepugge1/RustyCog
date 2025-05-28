use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex},
};

use crate::{
    cog::CogTrait, engine::Engine, error::CogError, machine::MachineMessage, types::CogType,
};

struct CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    cog: C,
    sender: crate::oneshot::Sender<Result<T, CogError>>,
}

impl<C, T> CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    fn new(cog: C, sender: crate::oneshot::Sender<Result<T, CogError>>) -> CogWrapper<C, T> {
        Self { cog, sender }
    }
}

impl<C, T> PartialEq for CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<C, T> Eq for CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
}

impl<C, T> PartialOrd for CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C, T> Ord for CogWrapper<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cog.priority(&other.cog)
    }
}

pub struct Dispatcher<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<C, T>>)>,
    receiver: std::sync::mpsc::Receiver<MachineMessage<C, T>>,
    ready_engines: Arc<(Mutex<usize>, Condvar)>,
    queue: BinaryHeap<CogWrapper<C, T>>,
}

impl<C, T> Dispatcher<C, T>
where
    C: CogTrait<T> + Send + 'static,
    T: CogType,
{
    pub fn new(
        engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<C, T>>)>,
        receiver: std::sync::mpsc::Receiver<MachineMessage<C, T>>,
        ready_engines: Arc<(Mutex<usize>, Condvar)>,
    ) -> Self {
        Self {
            engines,
            receiver,
            ready_engines,

            queue: BinaryHeap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            while let Ok(msg) = self.receiver.try_recv() {
                match msg {
                    MachineMessage::Work(cog, sender) => {
                        self.queue.push(CogWrapper::new(cog, sender))
                    }
                    MachineMessage::Terminate => self.terminate(),
                }
            }
            if let Some(cog) = self.queue.pop() {
                self.dispatch(cog);
            } else {
                match self.receiver.recv() {
                    Ok(MachineMessage::Work(cog, sender)) => {
                        self.queue.push(CogWrapper::new(cog, sender))
                    }
                    Ok(MachineMessage::Terminate) => self.terminate(),
                    Err(_e) => return,
                }
            }
        }
    }

    fn dispatch(&mut self, cog: CogWrapper<C, T>) {
        if *self.ready_engines.0.lock().unwrap() > 0 {
            for (engine, engine_sender) in self.engines.iter() {
                if engine.ready.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = engine_sender.send(MachineMessage::Work(cog.cog, cog.sender));
                    return;
                }
            }
        }
        self.queue.push(cog);
    }

    fn terminate(&self) {
        for (_engine, sender) in self.engines.iter() {
            let _ = sender.send(MachineMessage::Terminate);
        }
    }
}
