use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex},
};

use crate::{cog::CogTrait, engine::Engine, machine::MachineMessage};

struct CogWrapper<C>
where
    C: CogTrait + Send,
{
    cog: C,
    sender: crate::oneshot::Sender<C::T>,
}

impl<C> CogWrapper<C>
where
    C: CogTrait + Send + 'static,
{
    fn new(cog: C, sender: crate::oneshot::Sender<C::T>) -> CogWrapper<C> {
        Self { cog, sender }
    }
}

impl<C> PartialEq for CogWrapper<C>
where
    C: CogTrait + Send + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<C> Eq for CogWrapper<C> where C: CogTrait + Send + 'static {}

impl<C> PartialOrd for CogWrapper<C>
where
    C: CogTrait + Send + 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for CogWrapper<C>
where
    C: CogTrait + Send + 'static,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.cog.priority(&other.cog)
    }
}

pub struct Dispatcher<C>
where
    C: CogTrait + Send,
{
    engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<C>>)>,
    receiver: std::sync::mpsc::Receiver<MachineMessage<C>>,
    ready_engines: Arc<(Mutex<usize>, Condvar)>,
    queue: BinaryHeap<CogWrapper<C>>,
}

impl<C> Dispatcher<C>
where
    C: CogTrait + Send + 'static,
{
    pub fn new(
        engines: Vec<(Engine, std::sync::mpsc::Sender<MachineMessage<C>>)>,
        receiver: std::sync::mpsc::Receiver<MachineMessage<C>>,
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

    fn dispatch(&mut self, cog: CogWrapper<C>) {
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        Machine,
        cog::{CogId, PrioCog},
    };

    #[test]
    fn test_priorities() {
        let mut machine = Machine::cold(1);
        const COGS: CogId = 1000;

        let counter = Arc::new(Mutex::new(0));

        let mut cog_ids = Vec::new();

        for i in 0..COGS {
            let counter = counter.clone();
            cog_ids.push(machine.insert_cog(PrioCog::new(
                move || {
                    if *counter.lock().unwrap() == i {
                        *counter.lock().unwrap() += 1;
                        true
                    } else {
                        *counter.lock().unwrap() += 1;
                        false
                    }
                },
                COGS - i,
            )));
        }
        let _ = machine.power();

        for id in cog_ids {
            assert!(machine.wait_for_result(id).unwrap().unwrap());
        }
    }

    #[test]
    fn test_reverse_priorities() {
        let mut machine = Machine::cold(1);
        const COGS: CogId = 1000;

        let counter = Arc::new(Mutex::new(0));

        let mut cog_ids = Vec::new();

        for i in 0..COGS {
            let counter = counter.clone();
            cog_ids.push(machine.insert_cog(PrioCog::new(
                move || {
                    if *counter.lock().unwrap() == COGS - i - 1 {
                        *counter.lock().unwrap() += 1;
                        true
                    } else {
                        *counter.lock().unwrap() += 1;
                        false
                    }
                },
                i,
            )));
        }
        let _ = machine.power();

        for id in cog_ids {
            assert!(machine.wait_for_result(id).unwrap().unwrap());
        }
    }
}
