//! Each port's test, compiled in from the port's own folder.

use crate::harness::{Harness, exec, read, scratch};

include!(concat!(env!("OUT_DIR"), "/tests.rs"));
