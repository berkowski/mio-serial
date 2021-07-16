mod common;
use mio_serial::{self, SerialPort};
use std::convert::TryFrom;

#[test]
fn test_native_from_blocking() {
    let options = common::setup();

    let port_name = options.port_names[0].clone();
    let baud_rate = 9600;

    let native_blocking = mio_serial::new(port_name.clone(), baud_rate)
        .open_native()
        .expect(format!("Unable to open serial port named {}", port_name).as_str());
    let stream = mio_serial::SerialStream::try_from(native_blocking)
        .expect("Unable to open serial port in non-blocking mode");

    assert_eq!(stream.baud_rate().unwrap(), 9600);
}
