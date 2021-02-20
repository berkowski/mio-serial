//! Windows impl of mio-enabled serial ports.
use mio::{windows::NamedPipe, Interest, Registry, Token, event::Source};
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::Path;
use std::ptr;
use std::time::Duration;
use winapi::um::commapi::SetCommTimeouts;
use winapi::um::fileapi::*;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winbase::{COMMTIMEOUTS, FILE_FLAG_OVERLAPPED};
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE, HANDLE};

use crate::SerialPort;

/// Windows serial port
#[derive(Debug)]
pub struct COMPort {
    inner: serialport::COMPort,
    pipe: NamedPipe,
}

impl COMPort {
    /// Opens a COM port at the specified path
    pub fn open(builder: &crate::SerialPortBuilder) -> crate::Result<COMPort> {
        let (path, baud, parity, data_bits, stop_bits, flow_control) = {
            let com_port = serialport::COMPort::open(builder)?;
            let name = com_port.name().ok_or(crate::Error::new(
                crate::ErrorKind::NoDevice,
                "Empty device name",
            ))?;
            let baud = com_port.baud_rate()?;
            let parity = com_port.parity()?;
            let data_bits = com_port.data_bits()?;
            let stop_bits = com_port.stop_bits()?;
            let flow_control = com_port.flow_control()?;

            let mut path = Vec::<u16>::new();
            path.extend(OsStr::new("\\\\.\\").encode_wide());
            path.extend(Path::new(&name).as_os_str().encode_wide());
            path.push(0);

            (path, baud, parity, data_bits, stop_bits, flow_control)
        };



        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                0 as HANDLE,
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(
                crate::Error::from(
                io::Error::last_os_error()
                )
            );
        }
        let handle = unsafe { mem::transmute(handle) };

        // Construct NamedPipe and COMPort from Handle
        let pipe = unsafe { NamedPipe::from_raw_handle(handle) };
        let mut com_port = unsafe { serialport::COMPort::from_raw_handle(handle) };

        com_port.set_baud_rate(baud)?;
        com_port.set_parity(parity)?;
        com_port.set_data_bits(data_bits)?;
        com_port.set_stop_bits(stop_bits)?;
        com_port.set_flow_control(flow_control)?;
        override_comm_timeouts(handle)?;

        Ok(Self {
            inner: com_port,
            pipe: pipe,
        })
    }
}

impl crate::SerialPort for COMPort {

    /// Return the name associated with the serial port, if known.
    fn name(&self) -> Option<String> {
        self.inner.name()
    }

    /// Returns the current baud rate.
    ///
    /// This function returns `None` if the baud rate could not be determined. This may occur if
    /// the hardware is in an uninitialized state. Setting a baud rate with `set_baud_rate()`
    /// should initialize the baud rate to a supported value.
    fn baud_rate(&self) -> crate::Result<u32> {
        self.inner.baud_rate()
    }

    /// Returns the character size.
    ///
    /// This function returns `None` if the character size could not be determined. This may occur
    /// if the hardware is in an uninitialized state or is using a non-standard character size.
    /// Setting a baud rate with `set_char_size()` should initialize the character size to a
    /// supported value.
    fn data_bits(&self) -> crate::Result<crate::DataBits> {
        self.inner.data_bits()
    }

    /// Returns the flow control mode.
    ///
    /// This function returns `None` if the flow control mode could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported flow control
    /// mode. Setting a flow control mode with `set_flow_control()` should initialize the flow
    /// control mode to a supported value.
    fn flow_control(&self) -> crate::Result<crate::FlowControl> {
        self.inner.flow_control()
    }

    /// Returns the parity-checking mode.
    ///
    /// This function returns `None` if the parity mode could not be determined. This may occur if
    /// the hardware is in an uninitialized state or is using a non-standard parity mode. Setting
    /// a parity mode with `set_parity()` should initialize the parity mode to a supported value.
    fn parity(&self) -> crate::Result<crate::Parity> {
        self.inner.parity()
    }

    /// Returns the number of stop bits.
    ///
    /// This function returns `None` if the number of stop bits could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported stop bit
    /// configuration. Setting the number of stop bits with `set_stop-bits()` should initialize the
    /// stop bits to a supported value.
    fn stop_bits(&self) -> crate::Result<crate::StopBits> {
        self.inner.stop_bits()
    }

    /// Returns the current timeout.
    fn timeout(&self) -> Duration {
        Duration::from_secs(0)
    }

    // Port settings setters

    /// Applies all settings for a struct. This isn't guaranteed to involve only
    /// a single call into the driver, though that may be done on some
    /// platforms.
    // fn set_all(&mut self, settings: &SerialPortSettings) -> crate::Result<()> {
    //     self.inner.set_all(settings)?;
    //     override_comm_timeouts(self.inner.as_raw_handle())?;
    //     Ok(())
    // }

    /// Sets the baud rate.
    ///
    /// ## Errors
    ///
    /// If the implementation does not support the requested baud rate, this function may return an
    /// `InvalidInput` error. Even if the baud rate is accepted by `set_baud_rate()`, it may not be
    /// supported by the underlying hardware.
    fn set_baud_rate(&mut self, baud_rate: u32) -> crate::Result<()> {
        self.inner.set_baud_rate(baud_rate)
    }

    /// Sets the character size.
    fn set_data_bits(&mut self, data_bits: crate::DataBits) -> crate::Result<()> {
        self.inner.set_data_bits(data_bits)
    }

    /// Sets the flow control mode.
    fn set_flow_control(&mut self, flow_control: crate::FlowControl) -> crate::Result<()> {
        self.inner.set_flow_control(flow_control)
    }

    /// Sets the parity-checking mode.
    fn set_parity(&mut self, parity: crate::Parity) -> crate::Result<()> {
        self.inner.set_parity(parity)
    }

    /// Sets the number of stop bits.
    fn set_stop_bits(&mut self, stop_bits: crate::StopBits) -> crate::Result<()> {
        self.inner.set_stop_bits(stop_bits)
    }

    /// Sets the timeout for future I/O operations.  This parameter is ignored but
    /// required for trait completeness.
    fn set_timeout(&mut self, _: Duration) -> crate::Result<()> {
        Ok(())
    }

    // Functions for setting non-data control signal pins

    /// Sets the state of the RTS (Request To Send) control signal.
    ///
    /// Setting a value of `true` asserts the RTS control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the RTS control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn write_request_to_send(&mut self, level: bool) -> crate::Result<()> {
        self.inner.write_request_to_send(level)
    }

    /// Writes to the Data Terminal Ready pin
    ///
    /// Setting a value of `true` asserts the DTR control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the DTR control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn write_data_terminal_ready(&mut self, level: bool) -> crate::Result<()> {
        self.inner.write_data_terminal_ready(level)
    }

    // Functions for reading additional pins

    /// Reads the state of the CTS (Clear To Send) control signal.
    ///
    /// This function returns a boolean that indicates whether the CTS control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CTS control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_clear_to_send(&mut self) -> crate::Result<bool> {
        self.inner.read_clear_to_send()
    }

    /// Reads the state of the Data Set Ready control signal.
    ///
    /// This function returns a boolean that indicates whether the DSR control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the DSR control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_data_set_ready(&mut self) -> crate::Result<bool> {
        self.inner.read_data_set_ready()
    }

    /// Reads the state of the Ring Indicator control signal.
    ///
    /// This function returns a boolean that indicates whether the RI control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the RI control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_ring_indicator(&mut self) -> crate::Result<bool> {
        self.inner.read_ring_indicator()
    }

    /// Reads the state of the Carrier Detect control signal.
    ///
    /// This function returns a boolean that indicates whether the CD control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CD control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_carrier_detect(&mut self) -> crate::Result<bool> {
        self.inner.read_carrier_detect()
    }

    /// Gets the number of bytes available to be read from the input buffer.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn bytes_to_read(&self) -> crate::Result<u32> {
        self.inner.bytes_to_read()
    }

    /// Get the number of bytes written to the output buffer, awaiting transmission.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn bytes_to_write(&self) -> crate::Result<u32> {
        self.inner.bytes_to_write()
    }

    /// Discards all bytes from the serial driver's input buffer and/or output buffer.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> crate::Result<()> {
        self.inner.clear(buffer_to_clear)
    }

    // Misc methods

    /// Attempts to clone the `SerialPort`. This allow you to write and read simultaneously from the
    /// same serial connection. Please note that if you want a real asynchronous serial port you
    /// should look at [mio-serial](https://crates.io/crates/mio-serial) or
    /// [tokio-serial](https://crates.io/crates/tokio-serial).
    ///
    /// Also, you must be very carefull when changing the settings of a cloned `SerialPort` : since
    /// the settings are cached on a per object basis, trying to modify them from two different
    /// objects can cause some nasty behavior.
    ///
    /// # Errors
    ///
    /// This function returns an error if the serial port couldn't be cloned.
    fn try_clone(&self) -> crate::Result<Box<dyn crate::SerialPort>> {
        self.inner.try_clone()
    }

    #[inline(always)]
    fn set_break(&self) -> crate::Result<()> {
        self.inner.set_break()
    }

    #[inline(always)]
    fn clear_break(&self) -> crate::Result<()> {
        self.inner.clear_break()
    }
}

impl Read for COMPort {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.pipe.read(bytes)
    }
}

impl Write for COMPort {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pipe.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.pipe.flush()
    }
}

impl Source for COMPort {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> io::Result<()> {
        self.pipe.register(registry, token, interest)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interest: Interest,
    ) -> io::Result<()> {

        self.pipe.reregister(registry, token, interest)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        self.pipe.deregister(registry)
    }
}

/// Overrides timeout value set by serialport-rs so that the read end will
/// never wake up with 0-byte payload.
fn override_comm_timeouts(handle: RawHandle) -> io::Result<()> {
    let mut timeouts = COMMTIMEOUTS {
        // wait at most 1ms between two bytes (0 means no timeout)
        ReadIntervalTimeout: 1,
        // disable "total" timeout to wait at least 1 byte forever
        ReadTotalTimeoutMultiplier: 0,
        ReadTotalTimeoutConstant: 0,
        // write timeouts are just copied from serialport-rs
        WriteTotalTimeoutMultiplier: 0,
        WriteTotalTimeoutConstant: 0,
    };

    let r = unsafe { SetCommTimeouts(handle, &mut timeouts) };
    if r == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

impl crate::MioSerialPort for COMPort {
    #[inline(always)]
    fn source(&self) -> &dyn mio::event::Source {
        &self.pipe
    }
    #[inline(always)]
    fn source_mut(&mut self) -> &mut dyn mio::event::Source {
        &mut self.pipe
    }
}