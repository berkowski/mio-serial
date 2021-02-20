//! Simple example that echoes received serial traffic to stdout
extern crate mio;
extern crate mio_serial;

use mio::{Events, Interest, Poll, Token};

use std::env;
use std::io;
use std::io::Read;
use std::str;

use mio_serial::SerialPortBuilderExt;

const SERIAL_TOKEN: Token = Token(0);

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM6";

const DEFAULT_BAUD: u32 = 9600;

pub fn main() -> io::Result<()> {
    let mut args = env::args();
    let path = args.nth(1).unwrap_or(DEFAULT_TTY.into());
    // let baud = DEFAULT_BAUD;

    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events. Since we will only register a single serialport, a
    // capacity of 1 will do.
    let mut events = Events::with_capacity(1);

    // Create the serial port
    println!("Opening {} at 9600,8N1", path);
    let mut rx = mio_serial::new(path, DEFAULT_BAUD).open_async()?;

    // #[cfg(unix)]
    // let mut rx = mio_serial::TTYPort::open(&builder)?;
    // #[cfg(windows)]
    // let mut rx = mio_serial::COMPort::open(&builder)?;

    poll.registry()
        .register(&mut rx, SERIAL_TOKEN, Interest::READABLE)
        .unwrap();

    let mut buf = [0u8; 1024];

    loop {
        // Poll to check if we have events waiting for us.
        poll.poll(&mut events, None)?;

        // Process each event.
        for event in events.iter() {
            // Validate the token we registered our socket with,
            // in this example it will only ever be one but we
            // make sure it's valid none the less.
            match event.token() {
                SERIAL_TOKEN => loop {
                    // In this loop we receive all packets queued for the socket.
                    match rx.read(&mut buf) {
                        Ok(count) => {
                            println!("{:?}", String::from_utf8_lossy(&buf[..count]))
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            println!("Quitting due to read error: {}", e);
                            return Err(e);
                        }
                    }
                },
                _ => {
                    // This should never happen as we only registered our
                    // `UdpSocket` using the `UDP_SOCKET` token, but if it ever
                    // does we'll log it.
                    // warn!("Got event for unexpected token: {:?}", event);
                }
            }
        }
    }
}
