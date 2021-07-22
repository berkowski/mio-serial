mod common;
use mio::{Interest, Token};
use mio_serial::SerialPortBuilderExt;
use std::io::{Read, Write};

const TOKEN1: Token = Token(0);
const TOKEN2: Token = Token(1);

#[test]
fn test_builder_open_async() {
    let fixture = common::setup_virtual_serial_ports();
    let baud_rate = 9600;
    let builder = mio_serial::new(fixture.port_a, baud_rate);

    let stream = builder
        .open_native_async()
        .expect("unable to open serial port");

    common::assert_baud_rate(&stream, baud_rate)
}

#[test]
fn test_native_from_blocking() {
    use std::convert::TryFrom;
    let baud_rate = 9600;

    let fixture = common::setup_virtual_serial_ports();
    let port = fixture.port_a;
    let native_blocking = mio_serial::new(port, baud_rate)
        .open_native()
        .expect(format!("unable to open serial port {}", port).as_str());

    let stream = mio_serial::SerialStream::try_from(native_blocking)
        .expect("unable to convert from blocking serial port object");

    common::assert_baud_rate(&stream, baud_rate)
}

#[test]
fn test_stream_open() {
    let baud_rate = 9600;
    let fixture = common::setup_virtual_serial_ports();
    let port = fixture.port_a;
    let builder = mio_serial::new(port, baud_rate);
    let stream = mio_serial::SerialStream::open(&builder).expect("unable to open serial port");

    common::assert_baud_rate(&stream, baud_rate)
}

/// Port enumeration doesn't seem to work on virtual serial ports created by com0com during
/// the CI process
#[test]
#[ignore = "Port enumeration test does not seem to work with com0com virtual ports"]
fn test_port_enumeration() {
    let fixture = common::setup_virtual_serial_ports();
    let ports = mio_serial::available_ports().expect("unable to enumerate serial ports");
    for name in [fixture.port_a, fixture.port_b].iter() {
        ports.iter().find(|&info| info.port_name == *name).expect(
            format!(
                "unable to find serial port named {} in enumerated ports",
                name
            )
            .as_str(),
        );
    }
}

#[test]
fn test_read_write_pair() {
    let baud_rate = 38400;

    const DATA1: &[u8] = b"Here is an example string";
    const DATA2: &[u8] = b"And here is a reply to the example string";
    const DEFAULT_BUF_SIZE: usize = 64;

    let fixture = common::setup_virtual_serial_ports();
    let (port_a, port_b) = (fixture.port_a, fixture.port_b);
    let (mut poll, mut events) = common::init_with_poll();

    let mut port_1 = mio_serial::new(port_a, baud_rate)
        .open_native_async()
        .expect(format!("unable to open serial port {}", port_a).as_str());
    let mut port_2 = mio_serial::new(port_b, baud_rate)
        .open_native_async()
        .expect(format!("unable to open serial port {}", port_b).as_str());

    // register both serial ports for read and write events
    poll.registry()
        .register(&mut port_1, TOKEN1, Interest::WRITABLE | Interest::READABLE)
        .expect(
            format!(
                "unable to register port {} as readable and writable",
                port_a
            )
            .as_str(),
        );
    poll.registry()
        .register(&mut port_2, TOKEN2, Interest::READABLE | Interest::WRITABLE)
        .expect(
            format!(
                "unable to register port {} as readable and writable",
                port_b
            )
            .as_str(),
        );

    let mut buf = [0u8; DEFAULT_BUF_SIZE];

    // port1 should immediately be writable
    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN1, Interest::WRITABLE)],
    );

    // port2 should be blocking
    common::assert_would_block(port_2.read(&mut buf).into());

    // write data on port 1
    common::checked_write(&mut port_1, DATA1);
    port_1
        .flush()
        .expect(format!("unable to flush serial port {}", port_a).as_str());

    // port 2 should now be readable
    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN2, Interest::READABLE)],
    );

    // read data on port 2
    common::checked_read(&mut port_2, &mut buf, DATA1);

    // port 2 should then return to blocking
    common::assert_would_block(port_2.read(&mut buf));

    // port 1 should be blocking on read for the reply
    common::assert_would_block(port_1.read(&mut buf));

    // send data back on port 2
    common::checked_write(&mut port_2, DATA2);
    port_2
        .flush()
        .expect(format!("unable to flush serial port {}", port_b).as_str());

    // port 1 should now be readable
    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN1, Interest::READABLE)],
    );
    // and be able to read the full data
    common::checked_read(&mut port_1, &mut buf, DATA2);
    // .. before blocking again.
    common::assert_would_block(port_1.read(&mut buf));
}

// Same as test_send_recv but use a cloned receiver
#[cfg(never)]
#[test]
fn test_try_clone_native() {
    const DATA1: &[u8] = b"Here is an example string";
    const DEFAULT_BUF_SIZE: usize = 64;

    let baud_rate = 9600;
    let fixture = common::setup_virtual_serial_ports();
    let (mut poll, mut events) = common::init_with_poll();

    let builder_a = mio_serial::new(fixture.port_a, baud_rate);
    let builder_b = mio_serial::new(fixture.port_b, baud_rate);

    let mut sender =
        mio_serial::SerialStream::open(&builder_a).expect("unable to open serial port");
    let mut receiver =
        mio_serial::SerialStream::open(&builder_b).expect("unable to open serial port");

    // register the two ports
    poll.registry()
        .register(&mut sender, TOKEN1, Interest::WRITABLE | Interest::READABLE)
        .expect("unable to register port as readable and writable");
    poll.registry()
        .register(
            &mut receiver,
            TOKEN2,
            Interest::READABLE | Interest::WRITABLE,
        )
        .expect("unable to register port as readable and writable");

    // then clone one of them
    let mut cloned_receiver = receiver
        .try_clone_native()
        .expect("unable to clone serial port");

    // and drop the original
    std::mem::drop(receiver);

    let mut buf = [0u8; DEFAULT_BUF_SIZE];

    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN1, Interest::WRITABLE)],
    );

    common::assert_would_block(cloned_receiver.read(&mut buf).into());

    // write data on port 1
    common::checked_write(&mut sender, DATA1);
    sender.flush().expect("unable to flush serial port");

    // port 2 should now be readable
    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN2, Interest::READABLE)],
    );

    // read data on port 2
    common::checked_read(&mut cloned_receiver, &mut buf, DATA1);

    // port 2 should then return to blocking
    common::assert_would_block(cloned_receiver.read(&mut buf));

    // port 1 should be blocking on read for the reply
    common::assert_would_block(sender.read(&mut buf));

    // write data on port 1
    common::checked_write(&mut sender, DATA1);
    sender.flush().expect("unable to flush serial port");

    // port 2 should now be readable
    common::expect_events(
        &mut poll,
        &mut events,
        vec![common::ExpectEvent::new(TOKEN2, Interest::READABLE)],
    );

    // read data on port 2
    common::checked_read(&mut cloned_receiver, &mut buf, DATA1);

    // port 2 should then return to blocking
    common::assert_would_block(cloned_receiver.read(&mut buf));

    // port 1 should be blocking on read for the reply
    common::assert_would_block(sender.read(&mut buf));
}
