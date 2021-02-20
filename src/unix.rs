//! Unix impl of mio-enabled serial ports.
use std::io::{self, Read, Write};
use std::os::unix::prelude::*;
use std::time::Duration;
use std::convert::TryFrom;

use nix::sys::termios::{self, SetArg, SpecialCharacterIndices};
use nix::{self, libc};

use mio::unix::SourceFd;
use mio::{Registry, Token, Interest, event::Source};

/// *nix serial port using termios
#[derive(Debug)]
pub struct TTYPort {
    inner: serialport::TTYPort,
}

fn map_nix_error(e: nix::Error) -> crate::Error {
    crate::Error {
        kind: crate::ErrorKind::Io(io::ErrorKind::Other),
        description: e.to_string(),
    }
}

impl TTYPort {
    /// Open a nonblocking serial port from the provided builder
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use std::path::Path;
    ///
    /// let serial = TTYSerial::open(tty_name, 9600).unwrap();
    /// ```
    pub fn open(builder: &crate::SerialPortBuilder) -> crate::Result<TTYPort> {
        let tty = serialport::TTYPort::open(builder)?;
        TTYPort::try_from(tty)
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
    pub fn pair() -> crate::Result<(Self, Self)> {
        let (master, slave) = serialport::TTYPort::pair()?;

        let master = Self::try_from(master)?;
        let slave = Self::try_from(slave)?;

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
    pub fn set_exclusive(&mut self, exclusive: bool) -> crate::Result<()> {
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

impl crate::SerialPort for TTYPort {

    /// Start transmitting a break
    #[inline(always)]
    fn set_break(&self) -> crate::Result<()> {
        self.inner.set_break()
    }

    /// Stop transmitting a break
    #[inline(always)]
    fn clear_break(&self) -> crate::Result<()> {
        self.inner.clear_break()
    }

    /// Return the name associated with the serial port, if known.
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.inner.name()
    }

    /// Returns the current baud rate.
    ///
    /// This function returns `None` if the baud rate could not be determined. This may occur if
    /// the hardware is in an uninitialized state. Setting a baud rate with `set_baud_rate()`
    /// should initialize the baud rate to a supported value.
    #[inline(always)]
    fn baud_rate(&self) -> crate::Result<u32> {
        self.inner.baud_rate()
    }

    /// Returns the character size.
    ///
    /// This function returns `None` if the character size could not be determined. This may occur
    /// if the hardware is in an uninitialized state or is using a non-standard character size.
    /// Setting a baud rate with `set_char_size()` should initialize the character size to a
    /// supported value.
    #[inline(always)]
    fn data_bits(&self) -> crate::Result<crate::DataBits> {
        self.inner.data_bits()
    }

    /// Returns the flow control mode.
    ///
    /// This function returns `None` if the flow control mode could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported flow control
    /// mode. Setting a flow control mode with `set_flow_control()` should initialize the flow
    /// control mode to a supported value.
    #[inline(always)]
    fn flow_control(&self) -> crate::Result<crate::FlowControl> {
        self.inner.flow_control()
    }

    /// Returns the parity-checking mode.
    ///
    /// This function returns `None` if the parity mode could not be determined. This may occur if
    /// the hardware is in an uninitialized state or is using a non-standard parity mode. Setting
    /// a parity mode with `set_parity()` should initialize the parity mode to a supported value.
    #[inline(always)]
    fn parity(&self) -> crate::Result<crate::Parity> {
        self.inner.parity()
    }

    /// Returns the number of stop bits.
    ///
    /// This function returns `None` if the number of stop bits could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported stop bit
    /// configuration. Setting the number of stop bits with `set_stop-bits()` should initialize the
    /// stop bits to a supported value.
    #[inline(always)]
    fn stop_bits(&self) -> crate::Result<crate::StopBits> {
        self.inner.stop_bits()
    }

    /// Returns the current timeout. This parameter is const and equal to zero and implemented due
    /// to required for trait completeness.
    #[inline(always)]
    fn timeout(&self) -> Duration {
        Duration::from_secs(0)
    }

    // Port settings setters

    /// Sets the baud rate.
    ///
    /// ## Errors
    ///
    /// If the implementation does not support the requested baud rate, this function may return an
    /// `InvalidInput` error. Even if the baud rate is accepted by `set_baud_rate()`, it may not be
    /// supported by the underlying hardware.
    #[inline(always)]
    fn set_baud_rate(&mut self, baud_rate: u32) -> crate::Result<()> {
        self.inner.set_baud_rate(baud_rate)
    }

    /// Sets the character size.
    #[inline(always)]
    fn set_data_bits(&mut self, data_bits: crate::DataBits) -> crate::Result<()> {
        self.inner.set_data_bits(data_bits)
    }

    /// Sets the flow control mode.
    #[inline(always)]
    fn set_flow_control(&mut self, flow_control: crate::FlowControl) -> crate::Result<()> {
        self.inner.set_flow_control(flow_control)
    }

    /// Sets the parity-checking mode.
    #[inline(always)]
    fn set_parity(&mut self, parity: crate::Parity) -> crate::Result<()> {
        self.inner.set_parity(parity)
    }

    /// Sets the number of stop bits.
    #[inline(always)]
    fn set_stop_bits(&mut self, stop_bits: crate::StopBits) -> crate::Result<()> {
        self.inner.set_stop_bits(stop_bits)
    }

    /// Sets the timeout for future I/O operations. This parameter is ignored but
    /// required for trait completeness.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    fn clear(&self, buffer_to_clear: crate::ClearBuffer) -> crate::Result<()> {
        self.inner.clear(buffer_to_clear)
    }

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
    #[inline(always)]
    fn try_clone(&self) -> crate::Result<Box<dyn crate::SerialPort>> {
        self.inner.try_clone()
    }
}

impl TryFrom<serialport::TTYPort> for TTYPort {
    type Error = crate::Error;
    fn try_from(tty: serialport::TTYPort) -> Result<Self, Self::Error> {

        let mut t = termios::tcgetattr(tty.as_raw_fd()).map_err(map_nix_error)?;

        // Set VMIN = 1 to block until at least one character is received.
        t.control_chars[SpecialCharacterIndices::VMIN as usize] = 1;
        termios::tcsetattr(tty.as_raw_fd(), SetArg::TCSANOW, &t).map_err(map_nix_error)?;

        // Set the O_NONBLOCK flag.
        let flags = unsafe { libc::fcntl(tty.as_raw_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error().into());
        }

        match unsafe { libc::fcntl(tty.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } {
            0 => Ok(TTYPort { inner: tty }),
            _ => Err(io::Error::last_os_error().into()),
        }
    }
}

macro_rules! uninterruptibly {
    ($e:expr) => {{
        loop {
            match $e {
                Err(ref error) if error.kind() == io::ErrorKind::Interrupted => {}
                res => break res,
            }
        }
    }};
}

impl Read for TTYPort {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        })
    }
}

impl Write for TTYPort {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        uninterruptibly!(
            termios::tcdrain(self.inner.as_raw_fd()).map_err(|error| match error {
                nix::Error::Sys(errno) => io::Error::from(errno),
                error => io::Error::new(io::ErrorKind::Other, error.to_string()),
            })
        )
    }
}

impl<'a> Read for &'a TTYPort {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        })
    }
}

impl<'a> Write for &'a TTYPort {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        uninterruptibly!(
            termios::tcdrain(self.inner.as_raw_fd()).map_err(|error| match error {
                nix::Error::Sys(errno) => io::Error::from(errno),
                error => io::Error::new(io::ErrorKind::Other, error.to_string()),
            })
        )
    }
}

impl AsRawFd for TTYPort {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl IntoRawFd for TTYPort {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

impl FromRawFd for TTYPort {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let port = serialport::TTYPort::from_raw_fd(fd);
        TTYPort { inner: port }
    }
}

impl Source for TTYPort {

    #[inline(always)]
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).register(registry, token, interests)
    }

    #[inline(always)]
    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).reregister(registry, token, interests)
    }

    #[inline(always)]
    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        SourceFd(&self.as_raw_fd()).deregister(registry)
    }
}