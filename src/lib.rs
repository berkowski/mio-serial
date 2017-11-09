//! # mio-serial - Serial port I/O for mio
//!
//! This crate provides a serial port implementation compatable with mio.
//!
//! **At this time this crate ONLY provides a unix implementation**
//!
//! ## Links
//!   - repo:  https://github.com/berkowski/mio-serial
//!   - docs:  https://docs.rs/mio-serial
#![cfg(unix)]
#![deny(missing_docs)]

extern crate serialport;
extern crate mio;

#[cfg(unix)]
extern crate libc;
#[cfg(unix)]
extern crate termios;

// Enums, Structs, and Traits from the serialport crate
pub use serialport::{// Traits
                     SerialPort,

                     // Structs
                     Error,
                     SerialPortInfo,
                     SerialPortSettings,

                     // Enums
                     DataBits,
                     StopBits,
                     Parity,
                     BaudRate,
                     FlowControl};

// The serialport Result type, used in SerialPort trait.
pub use serialport::Result as SerialResult;

// Some enumeration functions from the serialport crate
pub use serialport::{available_baud_rates, available_ports};

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub use unix::Serial;
