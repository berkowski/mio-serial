use std::os::windows::prelude::*;
use std::io;
use std::path::Path;
use std::convert::AsRef;

use serialport::windows::COMport;

pub fn from_path<T: AsRef<Path>>(path: T, settings: &::SerialPortSettings) -> io::Result<::Serial> {


}

use std::os::windows::prelude::*;
use std::io::{self, Read, Write};
use std::path::Path;
use std::convert::AsRef;
use std::time::Duration;

use libc;
use mio::{Evented, PollOpt, Token, Poll, Ready};
use mio::unix::EventedFd;

use serialport;
use serialport::windows::COMPort;
use serialport::prelude::*;

pub struct Serial {
    inner: serialport::windows::COMPort,
}

impl Serial {
    pub fn from_path<T: AsRef<Path>>(path: T, settings: &SerialPortSettings) -> io::Result<Self> {
        COMPort
            .open(path.as_ref(), settings)
            .map(|port| ::Serial { inner: port })
            .map_err(|_| Err(io::Error::last_os_error))
    }
}

impl SerialPort for Serial {
    /// Returns a struct with the current port settings
    fn settings(&self) -> SerialPortSettings {
        self.inner.settings()
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
}

impl Read for Serial {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match unsafe {
                  libc::read(self.as_raw_fd(),
                             bytes.as_ptr() as *mut libc::c_void,
                             bytes.len() as libc::size_t)
              } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

impl Write for Serial {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match unsafe {
                  libc::write(self.as_raw_fd(),
                              bytes.as_ptr() as *const libc::c_void,
                              bytes.len() as libc::size_t)
              } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl AsRawFd for Serial {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Evented for Serial {
    fn register(&self,
                poll: &Poll,
                token: Token,
                interest: Ready,
                opts: PollOpt)
                -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self,
                  poll: &Poll,
                  token: Token,
                  interest: Ready,
                  opts: PollOpt)
                  -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}
