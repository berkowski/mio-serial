mod common;
use common::SerialPortTestExt;

use mio_serial;

#[test]
fn test_stream_open() {
    let baud_rate = 9600;
    common::with_virtual_serial_ports(|port, _| {
        let builder = mio_serial::new(port, baud_rate);
        let stream = mio_serial::SerialStream::open(&builder)?;

        stream.expect_baud_rate(baud_rate)
    })
}
