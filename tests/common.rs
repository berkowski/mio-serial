//! Common test code.  Adapted from `mio/tests/util/mod.rs`
#![allow(dead_code)]
use mio::{
    Events, Interest, Poll, Token,
    event::Event,
};
use std::io::{Read, Write};
use std::fmt::Formatter;
use std::time::Duration;
use std::ops::BitOr;

#[derive(Debug)]
pub enum MioError {
    ExpectedEvent {
        /// Expected events that were not found
        expected: Vec<ExpectEvent>,
    },
    WriteFailed(std::io::Error),
    ReadFailed(std::io::Error),
    ShortWrite{
        /// Expected number of bytes
        expected: usize,
        /// Actual number of bytes written,
        actual: usize,
    },
    ShortRead{
        /// Expected number of bytes
        expected: usize,
        /// Actual number of bytes read,
        actual: usize,
    },
    BadReadData{
        /// Expected data
        expected: Vec<u8>,
        /// Actual data read
        actual: Vec<u8>,
    },
    /// Expected to block but didn't
    ExpectedToBlock(Option<std::io::Error>)
}

impl std::fmt::Display for MioError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedEvent { ref expected } => {
                write!(f, "the following events were not found: {:?}", expected)
            }
            Self::WriteFailed(ref e) => write!(f, "failed to write data with error: {}", e),
            Self::ReadFailed(ref e) => write!(f, "failed to read data with error: {}", e),
            Self::ShortWrite {ref expected, ref actual} => write!(f, "wrote {} bytes instead of {}", actual, expected),
            Self::ShortRead {ref expected, ref actual} => write!(f, "read {} bytes instead of {}", actual, expected),
            Self::BadReadData {ref expected, ref actual} => write!(f, "mismatched data on read.  Expected {:?}, read {:?}", expected.as_slice(), actual.as_slice()),
            Self::ExpectedToBlock(ref maybe_error) => if let Some(e) = maybe_error {
                write!(f, "expected operation to block but errored with {} instead", e)
            }
            else {
                write!(f, "expected operation to block but it completed successfully.")
            }
        }
    }
}

impl std::error::Error for MioError {}

// impl Into<test::Error<MioError>> for MioError {
//     fn into(self) -> test::Error<MioError> {
//         test::Error::Other(self)
//     }
// }

impl From<MioError> for async_serial_test_helper::Error<MioError> {
    fn from(e: MioError) -> Self {
        Self::Other(e)
    }
}

pub type Error = async_serial_test_helper::Error<MioError>;

#[derive(Debug)]
pub struct Readiness(usize);

const READABLE: usize = 0b0000_0001;
const WRITABLE: usize = 0b0000_0010;
const AIO: usize = 0b0000_0100;
const LIO: usize = 0b0000_1000;
const ERROR: usize = 0b00010000;
const READ_CLOSED: usize = 0b0010_0000;
const WRITE_CLOSED: usize = 0b0100_0000;
const PRIORITY: usize = 0b1000_0000;

impl Readiness {
    pub const READABLE: Readiness = Readiness(READABLE);
    pub const WRITABLE: Readiness = Readiness(WRITABLE);
    pub const AIO: Readiness = Readiness(AIO);
    pub const LIO: Readiness = Readiness(LIO);
    pub const ERROR: Readiness = Readiness(ERROR);
    pub const READ_CLOSED: Readiness = Readiness(READ_CLOSED);
    pub const WRITE_CLOSED: Readiness = Readiness(WRITE_CLOSED);
    pub const PRIORITY: Readiness = Readiness(PRIORITY);

    fn matches(&self, event: &Event) -> bool {
        // If we expect a readiness then also match on the event.
        // In maths terms that is p -> q, which is the same  as !p || q.
        (!self.is(READABLE) || event.is_readable())
            && (!self.is(WRITABLE) || event.is_writable())
            && (!self.is(AIO) || event.is_aio())
            && (!self.is(LIO) || event.is_lio())
            && (!self.is(ERROR) || event.is_error())
            && (!self.is(READ_CLOSED) || event.is_read_closed())
            && (!self.is(WRITE_CLOSED) || event.is_write_closed())
            && (!self.is(PRIORITY) || event.is_priority())
    }

    /// Usage: `self.is(READABLE)`.
    fn is(&self, value: usize) -> bool {
        self.0 & value != 0
    }
}

impl BitOr for Readiness {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Readiness(self.0 | other.0)
    }
}

impl From<Interest> for Readiness {
    fn from(interests: Interest) -> Readiness {
        let mut readiness = Readiness(0);
        if interests.is_readable() {
            readiness.0 |= READABLE;
        }
        if interests.is_writable() {
            readiness.0 |= WRITABLE;
        }
        if interests.is_aio() {
            readiness.0 |= AIO;
        }
        if interests.is_lio() {
            readiness.0 |= LIO;
        }
        readiness
    }
}

pub fn init_with_poll() -> Result<(Poll, Events), Error> {
    let poll = Poll::new()?;
    let events = Events::with_capacity(16);
    Ok((poll, events))
}

/// An event that is expected to show up when `Poll` is polled, see
/// `expect_events`.
#[derive(Debug)]
pub struct ExpectEvent {
    token: Token,
    readiness: Readiness,
}

impl ExpectEvent {
    pub fn new<R>(token: Token, readiness: R) -> ExpectEvent
    where
        R: Into<Readiness>,
    {
        ExpectEvent {
            token,
            readiness: readiness.into(),
        }
    }

    fn matches(&self, event: &Event) -> bool {
        event.token() == self.token && self.readiness.matches(event)
    }
}

pub fn expect_events(
    poll: &mut Poll,
    events: &mut Events,
    mut expected: Vec<ExpectEvent>,
) -> Result<(), Error> {
    // In a lot of calls we expect more then one event, but it could be that
    // poll returns the first event only in a single call. To be a bit more
    // lenient we'll poll a couple of times.
    for _ in 0..3 {
        poll.poll(events, Some(Duration::from_millis(500)))?;

        for event in events.iter() {
            let index = expected.iter().position(|expected| expected.matches(event));

            if let Some(index) = index {
                expected.swap_remove(index);
            } else {
                // Must accept sporadic events.
                // warn!("got unexpected event: {:?}", event);
            }
        }

        if expected.is_empty() {
            return Ok(());
        }
    }

    if expected.is_empty() {
        return Ok(());
    }
    {
        return Err(Error::Other(MioError::ExpectedEvent { expected }));
    }
}

pub fn expect_block(result: std::io::Result<usize>) -> Result<(), Error> {
    match result {
        Ok(_) => Err(MioError::ExpectedToBlock(None).into()),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
        Err(e) => Err(MioError::ExpectedToBlock(Some(e)).into()),
    }
}
/// Ensure the entire buffer is written
pub fn checked_write(port: &mut mio_serial::SerialStream, data: &[u8]) -> Result<(), Error> {
    let n = port.write(data).map_err(|e| MioError::WriteFailed(e))?;
    if n == data.len() {
        Ok(())
    }
    else {
        Err(MioError::ShortWrite{expected: data.len(), actual: n}.into())
    }
}

/// Ensure the entire buffer is read
pub fn checked_read(port: &mut mio_serial::SerialStream, data: & mut [u8], expected: & [u8]) -> Result<(), Error> {

    let n = port.read(data).map_err(|e| MioError::ReadFailed(e))?;
    if n != expected.len() {
        return Err(MioError::ShortRead { expected: expected.len(), actual: n }.into());
    }

    if &data[..n] != expected {
        return Err(MioError::BadReadData {expected: Vec::from(expected), actual: Vec::from(&data[..n])}.into())
    }

    Ok(())
}

#[cfg(windows)]
fn setup_serial_ports(_: &str, _: &str) {}

#[cfg(windows)]
fn teardown_serial_ports(_:()){}

pub fn with_serial_ports<F, T>(test: F)
    where
        T: std::error::Error,
        F: FnOnce(&str, &str) -> Result<(), async_serial_test_helper::Error<T>>,
{
        async_serial_test_helper::with_virtual_serial_ports_setup_fixture(setup_serial_ports, test, teardown_serial_ports)
}
