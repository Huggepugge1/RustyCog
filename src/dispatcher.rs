use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

use crate::{cog::Cog, engine::Engine, machine::MachineMessage, types::CogType};

type CogFn<T> = Box<dyn FnOnce() -> T + Send + std::panic::UnwindSafe + 'static>;
type ShortCog<T> = Cog<T, CogFn<T>>;

pub struct Dispatcher<T: CogType> {
    engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<T>>)>,
    receiver: std::sync::mpsc::Receiver<MachineMessage<T>>,
    ready_engines: Arc<(Mutex<usize>, Condvar)>,
    queue: VecDeque<ShortCog<T>>,
}

impl<T: CogType> Dispatcher<T> {
    pub fn new(
        engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<T>>)>,
        receiver: std::sync::mpsc::Receiver<MachineMessage<T>>,
        ready_engines: Arc<(Mutex<usize>, Condvar)>,
    ) -> Self {
        Self {
            engines,
            receiver,
            ready_engines,

            queue: VecDeque::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            while let Ok(msg) = self.receiver.try_recv() {
                match msg {
                    MachineMessage::Work(cog) => self.queue.push_back(cog),
                    MachineMessage::Terminate => self.terminate(),
                }
            }
            if let Some(cog) = self.queue.pop_front() {
                self.dispatch(cog);
            } else {
                match self.receiver.recv() {
                    Ok(MachineMessage::Work(cog)) => self.queue.push_back(cog),
                    Ok(MachineMessage::Terminate) => self.terminate(),
                    Err(_e) => return,
                }
            }
        }
    }

    fn dispatch(&mut self, cog: ShortCog<T>) {
        if *self.ready_engines.0.lock().unwrap() > 0 {
            for (engine, sender) in self.engines.iter() {
                if engine.ready.load(std::sync::atomic::Ordering::SeqCst) {
                    let _ = sender.send(MachineMessage::Work(cog));
                    return;
                }
            }
        }
        self.queue.push_front(cog);
    }

    fn terminate(&self) {
        for (_engine, sender) in self.engines.iter() {
            let _ = sender.send(MachineMessage::Terminate);
        }
    }
}
