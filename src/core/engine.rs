use std::sync::{Arc, Weak};

use super::thread_pool::Pool;
use crate::traits::stream::*;

pub struct Engine {
    pipes: Vec<Arc<dyn base::Pipe>>,
    pool: Pool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            pipes: Vec::new(),
            pool: Pool::default(),
        }
        //         for _ in  0..4 {
        //
        //         }
    }

    fn insert_pipe<P>(&mut self, pipe: P) -> Weak<P>
    where
        P: base::Pipe,
    {
        let rpipe = Arc::new(pipe);
        let wref = Arc::downgrade(&rpipe);
        self.pipes.push(rpipe);
        wref
    }

    pub fn create_link(&mut self, from: &mut Arc<dyn base::Pipe>, into: &mut Arc<dyn base::Pipe>) {
        let stream = into.create_channel();
        from.add_output_channel(stream);
    }
}
