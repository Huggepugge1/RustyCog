use std::sync::{Arc, Condvar, Mutex};

use crate::error::RecvError;

pub fn channel<T: Send>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new((Mutex::new(None), Condvar::new()));
    let sender = Sender::new(inner.clone());
    let receiver = Receiver::new(inner);
    (sender, receiver)
}

pub struct Sender<T: Send> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T: Send> Sender<T> {
    fn new(inner: Arc<(Mutex<Option<T>>, Condvar)>) -> Self {
        Self { inner }
    }

    pub fn send(self, value: T) {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        *guard = Some(value);
        cvar.notify_one();
    }
}

pub struct Receiver<T: Send> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
    consumed: bool,
}

impl<T: Send> Receiver<T> {
    fn new(inner: Arc<(Mutex<Option<T>>, Condvar)>) -> Self {
        Self {
            inner,
            consumed: false,
        }
    }

    pub fn recv(self) -> Result<T, RecvError> {
        if self.consumed {
            return Err(RecvError::ValueConsumed);
        }
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cvar.wait(guard).unwrap();
        }
        Ok(guard.take().unwrap())
    }

    pub fn try_recv(&mut self) -> Option<T> {
        let (lock, _cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        match *guard {
            Some(ref _result) => {
                self.consumed = true;
                Some(guard.take().unwrap())
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_oneshot() {
        let (sender, mut receiver) = super::channel();
        assert_eq!(receiver.try_recv(), None);
        sender.send(1);
        assert_eq!(receiver.try_recv(), Some(1));
        assert_eq!(receiver.recv(), Err(crate::error::RecvError::ValueConsumed));
    }
}
