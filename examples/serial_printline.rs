//! Simple example that echos recevied serial traffic to stdout
extern crate mio;
extern crate mio_serial;

use mio::{Events, Poll, PollOpt, Ready, Token};
use mio::unix::UnixReady;
use std::io::Read;
use std::env;

const SERIAL_TOKEN: Token = Token(0);

pub fn main() {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // Create the listener
    let settings = mio_serial::SerialPortSettings::default();

    println!("Opening {} at 9600,8N1", tty_path);
    let mut rx = mio_serial::Serial::from_path(&tty_path, &settings).unwrap();

    // Disable exclusive mode
    rx.set_exclusive(false)
        .expect("Unable to set serial port into non-exclusive mode.");

    poll.register(
        &rx,
        SERIAL_TOKEN,
        Ready::readable() | UnixReady::hup() | UnixReady::error(),
        PollOpt::edge(),
    ).unwrap();

    let mut rx_buf = [0u8; 1024];

    'outer: loop {
        poll.poll(&mut events, None).unwrap();

        if events.is_empty() {
            println!("Read timed out!");
            continue;
        }

        for event in events.iter() {
            match event.token() {
                SERIAL_TOKEN => {
                    let ready = event.readiness();
                    if ready.contains(UnixReady::hup() | UnixReady::error()) {
                        println!("Quitting due to event: {:?}", ready);
                        break 'outer;
                    }
                    if ready.is_readable() {
                        match rx.read(&mut rx_buf) {
                            Ok(b) => match b {
                                b if b > 0 => {
                                    println!("{:?}", String::from_utf8_lossy(&rx_buf[..b]))
                                }
                                _ => println!("Read would have blocked."),
                            },
                            Err(e) => println!("Error:  {}", e),
                        }
                    }
                }
                t => unreachable!("Unexpected token: {:?}", t),
            }
        }
    }
}
