mod common;
use std::convert::TryFrom;

#[test]
fn test_native_from_blocking() {
    let baud_rate = 9600;

    common::with_serial_ports::<_, common::Error>(|port, _| {
        let native_blocking = mio_serial::new(port, baud_rate).open_native()?;

        let stream = mio_serial::SerialStream::try_from(native_blocking)?;

        async_serial_test_helper::expect_baud_rate(&stream, baud_rate)
    })
}
