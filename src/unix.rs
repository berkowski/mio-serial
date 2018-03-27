//! Unix impl of mio-enabled serial ports.
use std::os::unix::prelude::*;
use std::io::{self, Read, Write};
use std::path::Path;
use std::convert::AsRef;
use std::time::Duration;

use mio::{Evented, Poll, PollOpt, Ready, Token};
use mio::unix::EventedFd;

use serialport;
use serialport::posix::TTYPort;
use serialport::prelude::*;

use nix::libc;
use nix::sys::termios;
use nix::sys::termios::{SetArg, SpecialCharacterIndices};

/// *nix serial port using termios
pub struct Serial {
    inner: TTYPort,
}

impl Serial {
    /// Open a nonblocking serial port from the provided path.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use mio_serial::unix::Serial;
    /// use mio_serial::SerialPortSettings;
    ///
    /// let tty_name = Path::new("/dev/ttyUSB0");
    ///
    /// let serial = Serial::from_path(tty_name, &SerialPortSettings::default()).unwrap();
    /// ```
    pub fn from_path<T: AsRef<Path>>(path: T, settings: &SerialPortSettings) -> ::Result<Self> {
        let port = TTYPort::open(path.as_ref(), settings)?;
        Serial::from_serial(port)
    }

    /// Convert an existing `serialport::posix::TTYPort` struct.
    ///
    ///
    /// ## Example
    ///
    /// ```ignore
    /// extern crate serialport;
    ///
    /// use std::path::Path;
    /// use serialport::posix::TTYPort;
    /// use mio_serial::unix::Serial;
    ///
    /// let tty_name = Path::new("/dev/ttyUSB0");
    /// let blocking_serial = TTYPort::open(tty_path).unwrap();
    ///
    /// let serial = Serial::from_serial(blocking_serial).unwrap();
    /// # fn main() {}
    /// ```
    pub fn from_serial(port: TTYPort) -> ::Result<Self> {
        // Get the termios structure
        let mut t = termios::tcgetattr(port.as_raw_fd())?;

        // Set VMIN = 1 to block until at least one character is received.
        t.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;
        termios::tcsetattr(port.as_raw_fd(), SetArg::TCSANOW, &t)?;

        // Set the O_NONBLOCK flag.
        let flags = unsafe { libc::fcntl(port.as_raw_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error().into());
        }

        match unsafe { libc::fcntl(port.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } {
            0 => Ok(Serial { inner: port }),
            _ => Err(io::Error::last_os_error().into()),
        }
    }

    /// Create a pair of pseudo serial terminals
    ///
    /// ## Returns
    /// Two connected `Serial` objects: `(master, slave)`
    ///
    /// ## Errors
    /// Attempting any IO or parameter settings on the slave tty after the master
    /// tty is closed will return errors.
    ///
    /// ## Examples
    ///
    /// ```
    /// use mio_serial::unix::Serial;
    ///
    /// let (master, slave) = Serial::pair().unwrap();
    /// ```
    pub fn pair() -> ::Result<(Self, Self)> {
        let (master, slave) = TTYPort::pair()?;

        let master = Self::from_serial(master)?;
        let slave = Self::from_serial(slave)?;

        Ok((master, slave))
    }

    /// Sets the exclusivity of the port
    ///
    /// If a port is exclusive, then trying to open the same device path again
    /// will fail.
    ///
    /// See the man pages for the tiocexcl and tiocnxcl ioctl's for more details.
    ///
    /// ## Errors
    ///
    /// * `Io` for any error while setting exclusivity for the port.
    pub fn set_exclusive(&mut self, exclusive: bool) -> ::Result<()> {
        self.inner.set_exclusive(exclusive).map_err(|e| e)
    }

    /// Returns the exclusivity of the port
    ///
    /// If a port is exclusive, then trying to open the same device path again
    /// will fail.
    pub fn exclusive(&self) -> bool {
        self.inner.exclusive()
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
    fn try_clone(&self) -> serialport::Result<Box<SerialPort>> {
        self.inner.try_clone()
    }
}

impl Read for Serial {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

impl Write for Serial {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        termios::tcdrain(self.inner.as_raw_fd()).map_err(|e| {
            let e: ::Error = e.into();
            e.into()
        })
        //self.inner.flush()
    }
}

impl<'a> Read for &'a Serial {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

impl<'a> Write for &'a Serial {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        termios::tcdrain(self.inner.as_raw_fd()).map_err(|e| {
            let e: ::Error = e.into();
            e.into()
        })
    }
}

impl AsRawFd for Serial {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl IntoRawFd for Serial {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

impl FromRawFd for Serial {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let port = TTYPort::from_raw_fd(fd);
        Serial { inner: port }
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
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}
