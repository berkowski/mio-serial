//! # mio-serial - Serial port I/O for mio
//!
//! This crate provides a serial port implementation compatable with mio.
//!
//! **Windows support is present but largely untested by the author**
//!
//! ## Links
//!   - repo:  https://github.com/berkowski/mio-serial
//!   - docs:  https://docs.rs/mio-serial
#![deny(missing_docs)]
#![warn(rust_2018_idioms)]

// Enums, Structs, and Traits from the serialport crate
pub use serialport::{
    // Enums
    ClearBuffer,
    DataBits,
    StopBits,
    // Structs
    Error,
    ErrorKind,
    FlowControl,
    Parity,
    SerialPortInfo,
    SerialPortBuilder,
    // Traits
    SerialPort,
    // Types
    Result
};

// Re-export port-enumerating utility function.
pub use serialport::available_ports;

// Re-export creation of SerialPortBuilder objects
pub use serialport::new;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub use unix::TTYPort;

#[cfg(windows)]
pub use windows::Serial;

