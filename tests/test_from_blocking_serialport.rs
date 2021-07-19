mod common;
use std::convert::TryFrom;

#[test]
fn test_native_from_blocking() {
    let baud_rate = 9600;

    common::with_serial_ports(|port, _| {
        let native_blocking = mio_serial::new(port, baud_rate)
            .open_native()
            .expect(format!("unable to open serial port {}", port).as_str());

        let stream = mio_serial::SerialStream::try_from(native_blocking)
            .expect("unable to convert from blocking serial port object");

        common::assert_baud_rate(&stream, baud_rate)
    })
}
