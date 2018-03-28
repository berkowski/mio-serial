//! Windows impl of mio-enabled serial ports.
use std::io::{self, Read, Write};
use std::mem;
use std::path::Path;
use std::ptr;
use std::ffi::OsStr;
use std::time::Duration;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use winapi::um::fileapi::*;
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE, HANDLE};
use serialport::{self, SerialPort, SerialPortSettings};
use serialport::windows::COMPort;
use mio::{Evented, Poll, PollOpt, Ready, Token};
use mio_named_pipes::NamedPipe;


/// Windows serial port
pub struct Serial {
    inner: COMPort,
    pipe: NamedPipe,
}

impl Serial {
    /// Opens a COM port at the specified path
    pub fn from_path<T: AsRef<Path>>(path: T, settings: &SerialPortSettings) -> io::Result<Self> {
        let mut name = Vec::<u16>::new();

        name.extend(OsStr::new("\\\\.\\").encode_wide());
        name.extend(path.as_ref().as_os_str().encode_wide());
        name.push(0);

        let handle = unsafe {
            CreateFileW(name.as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        0,
                        ptr::null_mut(),
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                        0 as HANDLE)
        };

        if handle != INVALID_HANDLE_VALUE {
            let handle = unsafe { mem::transmute(handle) };

            // Construct NamedPipe and COMPort from Handle
            let pipe = unsafe { NamedPipe::from_raw_handle(handle) };
            let mut serial = unsafe { COMPort::from_raw_handle(handle) };
            serial.set_all(settings)?;

            Ok(Serial{
                inner: serial,
                pipe: pipe
            })
        } else {
            Err(io::Error::last_os_error())
        }

    }

}

impl SerialPort for Serial {
    /// Returns a struct with the current port settings
    fn settings(&self) -> SerialPortSettings {
        self.inner.settings()
    }

    /// Return the name associated with the serial port, if known.
    fn port_name(&self) -> Option<String> {
        self.inner.port_name()
    }

    /// Returns the current baud rate.
    ///
    /// This function returns `None` if the baud rate could not be determined. This may occur if
    /// the hardware is in an uninitialized state. Setting a baud rate with `set_baud_rate()`
    /// should initialize the baud rate to a supported value.
    fn baud_rate(&self) -> Option<::BaudRate> {
        self.inner.baud_rate()
    }

    /// Returns the character size.
    ///
    /// This function returns `None` if the character size could not be determined. This may occur
    /// if the hardware is in an uninitialized state or is using a non-standard character size.
    /// Setting a baud rate with `set_char_size()` should initialize the character size to a
    /// supported value.
    fn data_bits(&self) -> Option<::DataBits> {
        self.inner.data_bits()
    }

    /// Returns the flow control mode.
    ///
    /// This function returns `None` if the flow control mode could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported flow control
    /// mode. Setting a flow control mode with `set_flow_control()` should initialize the flow
    /// control mode to a supported value.
    fn flow_control(&self) -> Option<::FlowControl> {
        self.inner.flow_control()
    }

    /// Returns the parity-checking mode.
    ///
    /// This function returns `None` if the parity mode could not be determined. This may occur if
    /// the hardware is in an uninitialized state or is using a non-standard parity mode. Setting
    /// a parity mode with `set_parity()` should initialize the parity mode to a supported value.
    fn parity(&self) -> Option<::Parity> {
        self.inner.parity()
    }

    /// Returns the number of stop bits.
    ///
    /// This function returns `None` if the number of stop bits could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported stop bit
    /// configuration. Setting the number of stop bits with `set_stop-bits()` should initialize the
    /// stop bits to a supported value.
    fn stop_bits(&self) -> Option<::StopBits> {
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
    fn set_all(&mut self, settings: &SerialPortSettings) -> serialport::Result<()> {
        self.inner.set_all(settings)
    }

    /// Sets the baud rate.
    ///
    /// ## Errors
    ///
    /// If the implementation does not support the requested baud rate, this function may return an
    /// `InvalidInput` error. Even if the baud rate is accepted by `set_baud_rate()`, it may not be
    /// supported by the underlying hardware.
    fn set_baud_rate(&mut self, baud_rate: ::BaudRate) -> serialport::Result<()> {
        self.inner.set_baud_rate(baud_rate)
    }

    /// Sets the character size.
    fn set_data_bits(&mut self, data_bits: ::DataBits) -> serialport::Result<()> {
        self.inner.set_data_bits(data_bits)
    }

    /// Sets the flow control mode.
    fn set_flow_control(&mut self, flow_control: ::FlowControl) -> serialport::Result<()> {
        self.inner.set_flow_control(flow_control)
    }

    /// Sets the parity-checking mode.
    fn set_parity(&mut self, parity: ::Parity) -> serialport::Result<()> {
        self.inner.set_parity(parity)
    }

    /// Sets the number of stop bits.
    fn set_stop_bits(&mut self, stop_bits: ::StopBits) -> serialport::Result<()> {
        self.inner.set_stop_bits(stop_bits)
    }

    /// Sets the timeout for future I/O operations.  This parameter is ignored but
    /// required for trait completeness.
    fn set_timeout(&mut self, _: Duration) -> serialport::Result<()> {
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
    fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
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
    fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
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
    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
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
    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
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
    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
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
    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        self.inner.read_carrier_detect()
    }

    fn try_clone(&self) -> ::Result<Box<SerialPort>> {
        panic!("try_clone() is not supported");
    }
}

impl Read for Serial {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.pipe.read(bytes)
    }
}

impl Write for Serial {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pipe.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.pipe.flush()
    }
}

impl Evented for Serial {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.pipe.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.pipe.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.pipe.deregister(poll)
    }
}