//! Common test code.  Adapted from `mio/tests/util/mod.rs`
#![allow(dead_code)]
use mio::{event::Event, Events, Interest, Poll, Token};
use std::io::{Read, Write};
use std::ops::BitOr;
use std::panic;
use std::time::Duration;

use serialport::SerialPort;

#[cfg_attr(windows, allow(unused_imports))]
use std::process;
#[cfg_attr(windows, allow(unused_imports))]
use std::thread;

#[cfg(unix)]
const DEFAULT_PORT_NAMES: &'static str = "/tty/USB0;/tty/USB1";
#[cfg(windows)]
const DEFAULT_PORT_NAMES: &'static str = "COM1;COM2";

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
        "the following expected events were not found: {:?}",
        expected
    );
}

pub fn assert_would_block(result: std::io::Result<usize>) {
    match result {
        Ok(_) => panic!("unexpected OK result, expected a `WouldBlock` error"),
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        Err(e) => panic!("unexpected error result: {}", e),
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

#[cfg(windows)]
fn setup_serial_ports(_: &str, _: &str) {}

#[cfg(unix)]
fn setup_serial_ports(port_a: &str, port_b: &str) -> process::Child {
    let device_a = format!("PTY,link={}", port_a);
    let device_b = format!("PTY,link={}", port_b);
    let handle = process::Command::new("socat")
        .arg(device_a.as_str())
        .arg(device_b.as_str())
        .spawn()
        .expect("Unable to start socat process");
    println!("Socat started w/ id: '{}'", handle.id());
    thread::sleep(Duration::from_millis(500));
    handle
}

#[cfg(windows)]
fn teardown_serial_ports(_: ()) {}

#[cfg(unix)]
fn teardown_serial_ports(handle: process::Child) {
    let mut handle = handle;
    handle.kill().ok();
    handle.wait().ok();
}

pub fn with_serial_ports<F>(test: F)
where
    F: FnOnce(&str, &str) + panic::UnwindSafe,
{
    let port_names: Vec<String> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_PORT_NAMES)
        .split(';')
        .map(|s| s.to_owned())
        .collect();

    if port_names.len() < 2 {
        panic!("Expected two port names, found {}", port_names.len())
    }

    let port_a = port_names[0].as_str();
    let port_b = port_names[1].as_str();

    let fixture = setup_serial_ports(port_a, port_b);

    let result = std::panic::catch_unwind(|| test(port_a, port_b));

    teardown_serial_ports(fixture);

    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

pub fn assert_baud_rate<P>(port: &P, expected: u32)
    where
        P: SerialPort,
{
    let actual = port.baud_rate().expect("unable to get baud rate");

    assert_eq!(actual, expected, "baud rate not equal");
}
