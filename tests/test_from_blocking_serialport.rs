mod common;
use common::SerialPortTestExt;

use mio_serial;
use std::convert::TryFrom;

#[test]
fn test_native_from_blocking() {
    let baud_rate = 9600;

    common::with_virtual_serial_ports(|port, _| {
        let native_blocking = mio_serial::new(port, baud_rate).open_native()?;

        let stream = mio_serial::SerialStream::try_from(native_blocking)?;

        stream.expect_baud_rate(baud_rate)
    })
}
