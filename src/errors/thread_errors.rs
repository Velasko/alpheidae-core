use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ThreadExit {}

impl fmt::Display for ThreadExit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "ThreadExit")
    }
}

impl Error for ThreadExit {}
