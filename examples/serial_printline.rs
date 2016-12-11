//! Simple example that echos recevied serial traffic to stdout
extern crate mio;
extern crate mio_serial;

use mio::{Poll, PollOpt, Events, Token, Ready};
use std::time::Duration;
use std::str;

const SERIAL_TOKEN: Token = Token(0);

pub fn main() {
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // Create the listener
    let mut rx = mio_serial::SerialPort::open("/dev/ttyUSB0").unwrap();

    poll.register(&rx, SERIAL_TOKEN, Ready::readable(), PollOpt::level()).unwrap();

    let mut rx_buf = [0u8; 1024];

    loop {
        poll.poll(&mut events, Some(Duration::from_secs(1))).unwrap();

        if events.len() == 0 {
            println!("Read timed out!");
            continue;
        }

        for event in events.iter() {
            let bytes_read = match event.token() {
                SERIAL_TOKEN => rx.maybe_read(&mut rx_buf),
                _ => unreachable!(),
            };

            match bytes_read {
                Ok(b) => match b {
                    Some(x) => println!("{:?}", String::from_utf8_lossy(&rx_buf[..x])),
                    None => println!("Read would have blocked."),
                }, 
                Err(e) => println!("Error:  {}", e),
            }
        }
    }
}
