//! # mio-serial - A mio-compatable serial port implementation.
//!
//! This crate provides a serial port implementation compatable with mio.
//! 
//! ** At this time this crate ONLY provides a unix implementation **
//!
#![cfg(unix)]
#![deny(missing_docs)]

extern crate serialport;
extern crate mio;

#[cfg(unix)]
extern crate libc;
#[cfg(unix)]
extern crate termios;
#[cfg(unix)]
extern crate ioctl_rs;

pub use serialport::prelude::*;
pub use serialport::Result as SerialResult;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod widows;

#[cfg(unix)]
pub use unix::Serial;
