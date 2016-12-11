//! # mio-serial - A mio-compatable serial port implementation for *nix
//!
//! This crate provides a SerialPort implementation compatable with mio.
//! 
//! ** This crate ONLY provides a unix implementation **
//!
//! Some basic helper methods are provided for setting a few serial port
//! parameters such as the baud rate.  For everything else you'll
//! have to set the flags in the `termios::Termios` struct yourself!  All
//! the relavent settings can be found consulting your system's `man` page
//! for termios (e.g. `man termios`)
//!
//! This crate is influenced heavily by the [serial](https://github.com/dcuddeback/serial-rs)
//! crate (by David Cuddeback, same author of the helpful [termios](https://github.com/dcuddeback/termios-rs)
//! crate!)

#![cfg(unix)]
#![deny(missing_docs)]

extern crate termios;
extern crate libc;
extern crate mio;

use std::os::unix::prelude::*;
use std::io;
use std::ffi::CString;
use std::path::Path;
use std::convert::AsRef;

/// A mio compatable serial port for *nix
pub struct SerialPort {
    fd: RawFd,
    orig_settings:  termios::Termios,
    is_raw: bool,
}


impl SerialPort {

    /// Construct a new SerialPort
    ///
    /// Opens the a serial port at the location provided by `path` with the following
    /// default settings:
    ///
    ///   - 9600,8N1 (9600 Baud, 8-bit data, no parity, 1 stop bit)
    ///   - Receiver enabled in "Cannonical mode"
    ///   - Non-blocking
    ///   - No flow control (software OR hardware)
    ///   - Ignores hardware control lines
    ///
    /// # Errors
    ///
    /// SerialPort construction can fail for a few reasons:
    ///
    ///   -  An invalid path is provided
    ///   -  The path does not represent a serial port device
    ///   -  We are unable to configure the serial port 
    ///      ANY of the default settings. (Unlikely... but IS possible)
    pub fn open<T: AsRef<Path>>(path: T) -> io::Result<Self> {

        // Create a CString from the provided path.
        let path_cstr = CString::new(path.as_ref().as_os_str().as_bytes())
                            .map_err(|_| io::Error::last_os_error())?;

        // Attempt to open the desired path as a serial port.  Set it read/write, nonblocking, and
        // don't set it as the controlling terminal
        let fd = unsafe { libc::open(path_cstr.as_ptr(), libc::O_RDWR | libc::O_NONBLOCK | libc::O_NOCTTY, 0) };

        // Make sure the file descriptor is valid.
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        // Get the existing termios settings.  Close the file descriptor on errors.
        let orig_settings = termios::Termios::from_fd(fd).map_err(|e| unsafe {libc::close(fd); e})?;

        // Default port settings:  Cannonical 9600-8N1
        let mut default_settings = orig_settings.clone();
        default_settings.c_cflag = termios::CS8 | termios::CLOCAL | termios::CREAD;
        default_settings.c_oflag = 0;
        default_settings.c_iflag = termios::IGNPAR;
        default_settings.c_lflag = termios::ICANON;
        default_settings.c_cc[termios::VMIN] = 0;
        default_settings.c_cc[termios::VTIME] = 0;

        termios::cfsetspeed(&mut default_settings, termios::B9600).unwrap();

        // tcsetattr only errors out if we cannot set ANY attribute.  Something is seriously wrong
        // if that happens, so just close the file descriptor and raise the error.
        termios::tcsetattr(fd, termios::TCSANOW, &default_settings).map_err(|e| unsafe {libc::close(fd); e})?;

        Ok(SerialPort{
            fd: fd, 
            orig_settings: orig_settings,
            is_raw: false,
        })
    }

    /// Retrieve the termios structure for the serial port.
    pub fn termios(&self) -> io::Result<termios::Termios> {
        termios::Termios::from_fd(self.fd)
    }

    /// Set low-level serial port settings
    ///
    /// The `action` parameter must be one of the following:
    ///
    ///   - `termios::TCSANOW`      Update immediately
    ///   - `termios::TCSADRAIN`    Finish reading  buffered data before updating.
    ///   - `termios::TCSAFLUSH`    Finish writing  buffered data before updating.
    ///
    /// # Errors
    ///
    /// Will return `ErrorKind::InvalidInput` if `action` is not one of the three constants
    /// defined above.
    pub fn set_termios(&mut self, action: i32, t: &termios::Termios) -> io::Result<()> {
        match action {
            termios::TCSANOW | termios::TCSADRAIN | termios::TCSAFLUSH => {
                termios::tcsetattr(self.fd, action, t)
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Illegal action: {}", action))),
        }
    }


    /// Enable or disable blocking reads and writes.
    ///
    /// # Panics
    /// Will panic if the underlying `fcntl` system call returns a value other than 0 or -1
    pub fn set_nonblocking(&mut self, blocking: bool) -> io::Result<()> {

        match unsafe {libc::fcntl(self.fd, libc::F_SETFL, libc::O_NONBLOCK, blocking as libc::c_int)} {
            0 => Ok(()),
            -1 => Err(io::Error::last_os_error()),
            e @ _ => unreachable!(format!("Unexpected return code from F_SETFL O_NONBLOCK: {}", e)),
        }

    }

    /// Get the current blocking mode for the serial port
    ///
    /// # Panics
    /// Will panic if the underlying `fcntl` system call returns a value other than 0 or -1
    pub fn is_blocking(&self) -> io::Result<bool> {

        match unsafe {libc::fcntl(self.fd, libc::F_GETFL, libc::O_NONBLOCK)} {
            0 => Ok(false),
            1 => Ok(true),
            -1 => Err(io::Error::last_os_error()),
            e @ _ => unreachable!(format!("Unexpected return code from F_GETFL O_NONBLOCK: {}", e)),
        }

    }

    /// Try writing some data.
    ///
    /// Similar to the standard `io::Write` implementation, but errors
    /// due to blocking IO are translated into Ok(None) results.
    ///
    /// # Returns
    ///
    ///   - `Ok(Some(size))`  on successful writes
    ///   - `Ok(None)`        if calling write would block.
    ///   - `Err(e)`          for all other IO errors
    pub fn maybe_write(&mut self, buf: &[u8]) -> io::Result<Option<usize>> {

        match self.write(buf) {
            Ok(s) => Ok(Some(s)),
            Err(e) => {
                if let io::ErrorKind::WouldBlock = e.kind() {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }

    }

    /// Try reading some data.
    ///
    /// Similar to the standard `io::Read` implementation, but errors
    /// due to blocking IO are translated into Ok(None) results.
    ///
    /// # Returns
    ///
    ///   - `Ok(Some(size))`  on successful reads
    ///   - `Ok(None)`        if calling read would block.
    ///   - `Err(e)`          for all other IO errors
    pub fn maybe_read(&mut self, buf: &mut [u8]) -> io::Result<Option<usize>> {

        match self.read(buf) {
            Ok(s) => Ok(Some(s)),
            Err(e) => {
                if let io::ErrorKind::WouldBlock = e.kind() {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Set the serial baudrate
    ///
    /// Valid baudrates are:
    ///
    ///   -  0
    ///   -  50
    ///   -  75
    ///   -  110
    ///   -  134
    ///   -  150
    ///   -  200
    ///   -  300
    ///   -  600
    ///   -  1200
    ///   -  1800
    ///   -  2400
    ///   -  4800
    ///   -  9600
    ///   -  19200
    ///   -  38400
    ///
    /// # Errors
    ///
    /// Returns an io::ErrorKind::InvalidInput for baud rates no in the list
    /// above.
    pub fn set_baudrate(&mut self, baud: i32) -> io::Result<()> {
        use termios::{B0, B50, B75, B110, B134, B150, B200, B300, B600,
            B1200, B1800, B2400, B4800, B9600, B19200, B38400};

        let b = match baud {
            4800 => B4800,
            9600 => B9600,
            19200 => B19200,
            38400 => B38400,
            0 => B0,
            50 => B50,
            75 => B75,
            110 => B110,
            134 => B134,
            150 => B150,
            200 => B200,
            300 => B300,
            600 => B600,
            1200 => B1200,
            1800 => B1800,
            2400 => B2400,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{} is not a legal baudrate", baud))),
        };

        // Get the termios structure
        let mut s = self.termios()?;

        // And the original rate
        // let orig_rate = termios::cfgetospeed(&s);

        // Set the new rate
        termios::cfsetspeed(&mut s, b)?;

        // Now set the structure
        self.set_termios(termios::TCSAFLUSH, &s)
    }

    /// Get the serial baudrate
    ///
    /// Valid baudrates are:
    ///
    ///   -  0
    ///   -  50
    ///   -  75
    ///   -  110
    ///   -  134
    ///   -  150
    ///   -  200
    ///   -  300
    ///   -  600
    ///   -  1200
    ///   -  1800
    ///   -  2400
    ///   -  4800
    ///   -  9600
    ///   -  19200
    ///   -  38400
    ///
    /// # Errors
    ///
    /// Returns an io::ErrorKind::InvalidInput for baud rates no in the list
    /// above.
    pub fn baudrate(&self) -> io::Result<i32> {

        use termios::{B0, B50, B75, B110, B134, B150, B200, B300, B600,
            B1200, B1800, B2400, B4800, B9600, B19200, B38400};

        let s = self.termios()?;

        // And the original rate
        let baud = termios::cfgetospeed(&s);

        let b = match baud {
            B4800 => 4800,
            B9600 => 9600,
            B19200 => 19200,
            B38400 => 38400,
            B0 => 0,
            B50 => 50,
            B75 => 75,
            B110 => 110,
            B134 => 134,
            B150 => 150,
            B200 => 200,
            B300 => 300,
            B600 => 600,
            B1200 => 1200,
            B1800 => 1800,
            B2400 => 2400,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Unknown baud bitmask: {}", baud))),
        };
        
        Ok(b)
    }

    /// Enable or disable raw mode
    ///
    /// In raw mode, input is available character by character, echoing is disabled, and all
    /// special processing of terminal input and output characters is disabled.
    pub fn set_raw(&mut self, raw: bool) -> io::Result<()> {

        if raw == self.is_raw() {
            return Ok(())
        }

        let mut s = self.termios()?;
        
        if raw {
            termios::cfmakeraw(&mut s);
        } else {
            s.c_iflag |= termios::IGNBRK | termios::PARMRK;
            s.c_lflag |= termios::ICANON;
        }

        self.set_termios(termios::TCSANOW, &s)?;

        self.is_raw = raw;
        Ok(())
    }

    /// Return if raw mode is enabled or not.
    pub fn is_raw(&self) -> bool {
        self.is_raw
    }
}

impl Drop for SerialPort {

    fn drop(&mut self) {
        
        #[allow(unused_must_use)]
        unsafe {
            // Reset termios settings to their original state.
            let s = self.orig_settings.clone();
            self.set_termios(termios::TCSANOW, &s);

            // Close the file descriptor
            libc::close(self.fd);
        }
    }
}


impl AsRawFd for SerialPort {

    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

}

use std::io::Read;
impl Read for SerialPort {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match unsafe {libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())} {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }

}

use std::io::Write;
impl Write for SerialPort {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match unsafe {libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len())} {
            x if x >= 0 => Ok(x as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        termios::tcflush(self.fd, termios::TCOFLUSH)
    }
}


use mio::{Evented, PollOpt, Token, Poll, Ready};
use mio::unix::EventedFd;
impl Evented for SerialPort {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}
