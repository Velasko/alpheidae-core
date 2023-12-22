use crate::errors::thread_errors::ThreadExit;
use std::{
    any::Any,
    cell::RefCell,
    collections::{hash_map::HashMap, vec_deque::VecDeque},
    ops::Deref,
    panic::{catch_unwind, UnwindSafe},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

pub struct Queue<T> {
    data: Mutex<RefCell<VecDeque<T>>>,
    pop_lock: Condvar,
    timeout: Duration,
}

impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(RefCell::new(VecDeque::new())),
            pop_lock: Condvar::new(),
            timeout: Duration::from_millis(100),
        }
    }

    pub fn push(&self, data: T) {
        let mut binding = self.data.lock().unwrap();
        let queue = (*binding).get_mut();
        queue.push_back(data);
        self.pop_lock.notify_one();
    }

    pub fn pop(&self) -> T {
        let data = {
            loop {
                let lock = self.data.lock().unwrap();
                let (mut binding, _) = self.pop_lock.wait_timeout(lock, self.timeout).unwrap();
                let queue = (*binding).get_mut();
                if let Some(item) = queue.pop_front() {
                    break item;
                }
            }
        };
        self.pop_lock.notify_one();
        data
    }

    pub fn notify_one(&self) {
        self.pop_lock.notify_one()
    }

    pub fn notify_all(&self) {
        self.pop_lock.notify_all()
    }
}

unsafe impl<T> Sync for Queue<T> {}
unsafe impl<T> Send for Queue<T> {}

fn thread_operation(queue: Arc<Queue<Box<dyn FnOnce() -> anyhow::Result<()>>>>) {
    loop {
        let func = queue.pop();
        let _ = func();
    }
}

pub struct Pool {
    queue: Arc<Queue<Box<dyn FnOnce() -> anyhow::Result<()>>>>,
    threads: Vec<thread::JoinHandle<()>>,
    daemon: bool,
}

impl Pool {
    pub fn new(thread_ammount: usize, daemon: bool) -> Self {
        let queue = Arc::new(Queue::new());
        let threads = (0..thread_ammount)
            .map(|_| {
                let rqueue = Arc::clone(&queue);
                thread::spawn(move || thread_operation(rqueue))
            })
            .collect::<Vec<thread::JoinHandle<()>>>();
        Pool {
            queue,
            threads,
            daemon,
        }
    }

    pub fn default() -> Self {
        let core_count = match sysinfo::System::new().physical_core_count() {
            Some(count) => count,
            None => 1,
        };
        Self::new(core_count, false)
    }

    pub fn star_map<T: 'static + UnwindSafe, S: 'static>(
        &self,
        func: fn(T) -> S,
        args: Vec<T>,
    ) -> Vec<Result<S, Box<dyn Any + Send>>> {
        let map = Arc::new(Mutex::new(RefCell::new(HashMap::new())));

        let func = Arc::new(func);

        let args_size = args.len();

        for (n, arg) in args.into_iter().enumerate() {
            let rmap = Arc::clone(&map);
            let rfunc = Arc::clone(&func);
            let lambda = move || -> anyhow::Result<()> {
                let ret = catch_unwind(|| rfunc(arg));
                loop {
                    if let Ok(mut return_map) = rmap.lock() {
                        return_map.get_mut().insert(n, ret);
                        return Ok(());
                    };
                }
            };
            self.queue.push(Box::new(lambda));
        }

        loop {
            match map.lock() {
                Err(_) => (),
                Ok(return_map) => {
                    if return_map.deref().borrow().len() >= args_size {
                        let mut rmap = return_map.take();
                        return (0..args_size)
                            .map(|key| rmap.remove(&key).unwrap())
                            .collect::<Vec<Result<S, Box<dyn Any + Send>>>>();
                    };
                }
            }
        }
    }
}
