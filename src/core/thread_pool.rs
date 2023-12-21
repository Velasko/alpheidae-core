use std::cell::RefCell;
use std::collections::vec_deque::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;

pub struct Queue<T> {
    data: Mutex<RefCell<VecDeque<T>>>,
    pop_lock: Condvar,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Self {
            data: Mutex::new(RefCell::new(VecDeque::new())),
            pop_lock: Condvar::new(),
        }
    }

    fn push(&self, data: T) {
        let queue = (*self.data.lock().unwrap()).get_mut();
        queue.push_back(data);
        self.pop_lock.notify_one();
    }

    fn pop(&self) -> T {
        let mut queue = (*self.pop_lock.wait(self.data.lock().unwrap()).unwrap()).get_mut();
        while queue.len() < 1 {
            queue = (*self.pop_lock.wait(self.data.lock().unwrap()).unwrap()).get_mut();
        }
        self.pop_lock.notify_one();

        queue.pop_front().unwrap()
    }
}

pub struct Pool {
    queue: Queue<Box<dyn FnOnce() -> anyhow::Result<()>>>,
    threads: Vec<thread::Thread>,
}

impl Pool {
    pub fn new(thread_ammount: i32) -> Self {
        Pool {
            queue: Queue::new(),
            threads: Vec::new(),
        }
    }

    pub fn map<T, S>(&mut self, func: fn(T) -> S, args: Vec<T>) -> Vec<S> {
        let vec = Mutex::new(Vec::new());
        let args_size = args.len();
        for arg in args.into_iter() {
            self.queue.push(Box::new(|| -> anyhow::Result<()> {
                let ret = func(arg);
                vec.lock().unwrap().push(ret);
                Ok(())
            }));
        }

        while vec.lock().unwrap().len() < args_size {}
        vec.into_inner().unwrap()
    }
}
