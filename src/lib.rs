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

use mio::{Interest, Registry, Token};

// Enums, Structs, and Traits from the serialport crate
pub use serialport::{
    // Enums
    ClearBuffer,
    DataBits,
    // Structs
    Error,
    ErrorKind,
    FlowControl,
    Parity,
    // Types
    Result,
    // Traits
    SerialPort,
    SerialPortBuilder,
    SerialPortInfo,
    StopBits,
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
pub use windows::COMPort;

/// An extension trait for SerialPortBuilder
///
/// This trait adds two methods to SerialPortBuilder:
///
/// - open_async
/// - open_native_async
///
/// These methods mirror the `open` and `open_native` methods of SerialPortBuilder
pub trait SerialPortBuilderExt {
    /// Open a cross-platform interface to the port with the specified settings
    fn open_async(self) -> Result<Box<dyn MioSerialPort>>;

    #[cfg(unix)]
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> Result<TTYPort>;

    #[cfg(windows)]
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> Result<COMPort>;
}

impl SerialPortBuilderExt for SerialPortBuilder {
    /// Open a cross-platform interface to the port with the specified settings
    fn open_async(self) -> Result<Box<dyn MioSerialPort>> {
        #[cfg(unix)]
        return TTYPort::open(&self).map(|p| Box::new(p) as Box<dyn SerialPort>);

        #[cfg(windows)]
        return COMPort::open(&self).map(|p| Box::new(p) as Box<dyn MioSerialPort>);

        #[cfg(not(any(unix, windows)))]
        Err(Error::new(
            ErrorKind::Unknown,
            "open() not implemented for platform",
        ))
    }
    #[cfg(unix)]
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> Result<TTYPort> {
        TTYPort::open(&self)
    }

    #[cfg(windows)]
    /// Open a platform-specific interface to the port with the specified settings
    fn open_native_async(self) -> Result<COMPort> {
        COMPort::open(&self)
    }
}

/// An async, platform independent serial port
pub trait MioSerialPort: private::Sealed + SerialPort {
    /// Get access to the underlying MIO Source trait object
    fn source(&self) -> &dyn mio::event::Source;
    /// Get mutable access to the underlying MIO Source trait object
    fn source_mut(&mut self) -> &mut dyn mio::event::Source;
}

impl mio::event::Source for dyn MioSerialPort {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> std::io::Result<()> {
        self.source_mut().register(registry, token, interest)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> std::io::Result<()> {
        self.source_mut().reregister(registry, token, interest)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        self.source_mut().deregister(registry)
    }
}
mod private {
    pub trait Sealed {}
    #[cfg(unix)]
    impl Sealed for crate::TTYPort {}

    #[cfg(windows)]
    impl Sealed for crate::COMPort {}
}
