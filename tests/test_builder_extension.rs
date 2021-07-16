mod common;

use mio_serial::{self, SerialPort, SerialPortBuilderExt};

#[test]
fn test_builder_open_async() {
    let options = common::setup();
    let port_name = options.port_names[0].clone();
    let builder = mio_serial::new(port_name.clone(), 9600);

    let stream = builder.open_async().unwrap();

    assert_eq!(stream.baud_rate().unwrap(), 9600);
}
