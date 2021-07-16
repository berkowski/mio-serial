mod common;

use mio_serial::{self, SerialPort};

#[test]
fn test_stream_open() {
    let options = common::setup();
    let port_name = options.port_names[0].clone();
    let builder = mio_serial::new(port_name.clone(), 9600);
    let stream = mio_serial::SerialStream::open(&builder)
        .expect(format! {"Unable to open serial port {}", port_name}.as_str());

    assert_eq!(stream.baud_rate().unwrap(), 9600)
}
