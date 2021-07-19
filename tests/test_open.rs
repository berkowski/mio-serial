mod common;

#[test]
fn test_stream_open() {
    let baud_rate = 9600;
    common::with_serial_ports(|port, _| {
        let builder = mio_serial::new(port, baud_rate);
        let stream = mio_serial::SerialStream::open(&builder).expect("unable to open serial port");

        async_serial_test_helper::assert_baud_rate(&stream, baud_rate)
    })
}
