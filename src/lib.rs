//! # mio-serial - A mio-compatable serial port implementation.
//!
//! This crate provides a serial port implementation compatable with mio.
//! 
//! ** At this time this crate ONLY provides a unix implementation **
//!
#![deny(missing_docs)]

extern crate serialport;
extern crate mio;

#[cfg(unix)]
extern crate libc;

#[cfg(unix)]
pub mod posix;

pub use serialport::{BaudRate, FlowControl, DataBits, StopBits, Parity, SerialPort, SerialPortSettings};

