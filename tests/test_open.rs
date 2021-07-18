mod common;

#[test]
fn test_stream_open() {
    let baud_rate = 9600;
    common::with_serial_ports::<_, std::convert::Infallible>(|port, _| {
        let builder = mio_serial::new(port, baud_rate);
        let stream = mio_serial::SerialStream::open(&builder)?;

        async_serial_test_helper::expect_baud_rate(&stream, baud_rate)

        //<SerialStream as SerialPortTestExt<common::Error>>::expect_baud_rate(baud_rate)
        //SerialPortTestExt::expect_baud_rate(&<stream as SerialPort>, baud_rate)
    })
}
