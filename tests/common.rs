//! Common test code.  Adapted from `mio/tests/util/mod.rs`
#![allow(dead_code)]
use mio::{event::Event, Events, Interest, Poll, Token};
use std::io::{Read, Write};
use std::ops::BitOr;
use std::panic;
use std::sync::Once;
use std::time::Duration;

use serialport::SerialPort;

#[cfg(unix)]
use std::process;
#[cfg(unix)]
use std::thread;

static LOGGING_INIT: Once = Once::new();

/// Default serial port names used for testing
#[cfg(unix)]
const DEFAULT_TEST_PORT_NAMES: &str = "USB0;USB1";

/// Default serial port names used for testing
#[cfg(windows)]
const DEFAULT_TEST_PORT_NAMES: &str = "COM1;COM2";

#[derive(Debug)]
pub struct Readiness(usize);

const READABLE: usize = 0b0000_0001;
const WRITABLE: usize = 0b0000_0010;
const AIO: usize = 0b0000_0100;
const LIO: usize = 0b0000_1000;
const ERROR: usize = 0b0001_0000;
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

#[must_use]
pub fn init_with_poll() -> (Poll, Events) {
    let poll = Poll::new().expect("unable to create poll object");
    let events = Events::with_capacity(16);
    (poll, events)
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

pub fn expect_events(poll: &mut Poll, events: &mut Events, mut expected: Vec<ExpectEvent>) {
    // In a lot of calls we expect more then one event, but it could be that
    // poll returns the first event only in a single call. To be a bit more
    // lenient we'll poll a couple of times.
    for _ in 0..3 {
        poll.poll(events, Some(Duration::from_millis(500)))
            .expect("unable to poll");

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
            return;
        }
    }

    assert!(
        expected.is_empty(),
        "the following expected events were not found: {expected:?}",
    );
}

pub fn assert_would_block(result: std::io::Result<usize>) {
    match result {
        Ok(_) => panic!("unexpected OK result, expected a `WouldBlock` error"),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        Err(e) => panic!("unexpected error result: {e}"),
    }
}

/// Ensure the entire buffer is written
pub fn checked_write(port: &mut mio_serial::SerialStream, data: &[u8]) {
    let n = port.write(data).expect("unable to write to serial port");
    assert_eq!(n, data.len(), "short write");
}

/// Ensure the entire buffer is read
pub fn checked_read(port: &mut mio_serial::SerialStream, data: &mut [u8], expected: &[u8]) {
    let n = port.read(data).expect("unable to read from serial port");
    assert_eq!(n, expected.len(), "short read");
    assert_eq!(&data[..n], expected);
}

pub struct Fixture {
    #[cfg(unix)]
    process: process::Child,
    pub port_a: &'static str,
    pub port_b: &'static str,
}

#[cfg(unix)]
impl Drop for Fixture {
    fn drop(&mut self) {
        log::trace!("stopping socat process (id: {})...", self.process.id());
        self.process.kill().ok();
        thread::sleep(Duration::from_millis(1000));
        log::trace!("removing link: {:?}", self.port_a);
        std::fs::remove_file(&self.port_a).ok();
        log::trace!("removing link: {:?}", self.port_b);
        std::fs::remove_file(&self.port_b).ok();
        thread::sleep(Duration::from_millis(1000));
    }
}

impl Fixture {
    #[cfg(unix)]
    pub fn new(port_a: &'static str, port_b: &'static str) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        LOGGING_INIT.call_once(|| env_logger::init());
        let n = N.fetch_add(1, Ordering::Relaxed);
        let port_a = format!("{}{}", port_a, n).leak();
        let port_b = format!("{}{}", port_b, n).leak();
        let args = [
            format!("PTY,link={}", port_a),
            format!("PTY,link={}", port_b),
        ];
        log::trace!("starting process: socat {} {}", args[0], args[1]);

        let process = process::Command::new("socat")
            .args(&args)
            .spawn()
            .expect("unable to spawn socat process");
        log::trace!(".... done! (pid: {:?})", process.id());
        thread::sleep(Duration::from_millis(1000));
        Self {
            process,
            port_a,
            port_b,
        }
    }

    #[cfg(not(unix))]
    pub fn new(port_a: &'static str, port_b: &'static str) -> Self {
        LOGGING_INIT.call_once(|| env_logger::init());
        Self { port_a, port_b }
    }
}

pub fn setup_virtual_serial_ports() -> Fixture {
    let port_names: Vec<&str> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_TEST_PORT_NAMES)
        .split(';')
        .collect();

    assert_eq!(port_names.len(), 2);
    Fixture::new(port_names[0], port_names[1])
}

/// Assert serial port baud rate matches expected value.
pub fn assert_baud_rate<P>(port: &P, expected: u32)
where
    P: SerialPort,
{
    let actual = port.baud_rate().expect("unable to get baud rate");

    assert_eq!(actual, expected, "baud rate not equal");
}
