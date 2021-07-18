mod common;

use mio::{
    Interest, Token,
};
use mio_serial::SerialPortBuilderExt;

use std::io::{Read, Write};

const DATA1: &[u8] = b"Here is an example string";
const DATA2: &[u8] = b"And here is a reply to the example string";
//const DATA1_LEN: usize = DATA1.len();
const DEFAULT_BUF_SIZE: usize = 64;
const TOKEN1: Token = Token(0);
const TOKEN2: Token = Token(1);

#[test]
fn test_read_write_pair() {
    let baud_rate = 38400;
    common::with_serial_ports(|port_a, port_b| {
        let (mut poll, mut events) = common::init_with_poll()?;

        let mut port_1 = mio_serial::new(port_a, baud_rate).open_native_async()?;
        let mut port_2 = mio_serial::new(port_b, baud_rate).open_native_async()?;

        // register both serial ports for read and write events
        poll.registry()
            .register(&mut port_1, TOKEN1, Interest::WRITABLE | Interest::READABLE)?;
        poll.registry()
            .register(&mut port_2, TOKEN2, Interest::READABLE | Interest::WRITABLE)?;

        let mut buf = [0u8; DEFAULT_BUF_SIZE];

        // port1 should immediately be writable
        common::expect_events(
            &mut poll,
            &mut events,
            vec![
                common::ExpectEvent::new(TOKEN1, Interest::WRITABLE),
            ],
        )?;

        // port2 should be blocking
        common::expect_block(port_2.read(&mut buf).into())?;

        // write data on port 1
        common::checked_write(&mut port_1, DATA1)?;
        port_1.flush()?;

        // port 2 should now be readable
        common::expect_events(
            &mut poll,
            &mut events,
            vec![
                common::ExpectEvent::new(TOKEN2, Interest::READABLE),
            ],
        )?;

        // read data on port 2
        common::checked_read(&mut port_2, &mut buf, DATA1)?;

        // port 2 should then return to blocking
        common::expect_block(port_2.read(&mut buf).into())?;


        // port 1 should be blocking on read for the reply
        common::expect_block(port_1.read(&mut buf).into())?;

        // send data back on port 2
        common::checked_write(&mut port_2, DATA2)?;
        port_2.flush()?;

        // port 1 should now be readable
        common::expect_events(
            &mut poll,
            &mut events,
            vec![
                common::ExpectEvent::new(TOKEN1, Interest::READABLE),
            ],
        )?;
        // and be able to read the full data
        common::checked_read(&mut port_1, &mut buf, DATA2)?;
        // .. before blocking again.
        common::expect_block(port_1.read(&mut buf).into())?;
        Ok(())
    })
}
