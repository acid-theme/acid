//! Each port's preview, compiled in from the port's own folder.

use crate::preview::{FONT, Preview};

include!(concat!(env!("OUT_DIR"), "/previews.rs"));
