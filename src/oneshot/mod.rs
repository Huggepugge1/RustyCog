use std::sync::{Arc, Condvar, Mutex};

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new((Mutex::new(None), Condvar::new()));
    let sender = Sender::new(inner.clone());
    let receiver = Receiver::new(inner);
    (sender, receiver)
}

pub struct Receiver<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> Receiver<T> {
    fn new(inner: Arc<(Mutex<Option<T>>, Condvar)>) -> Self {
        Self { inner }
    }

    pub fn recv(self) -> T {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        while guard.is_none() {
            guard = cvar.wait(guard).unwrap();
        }
        guard.take().unwrap()
    }
}

pub struct Sender<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> Sender<T> {
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
