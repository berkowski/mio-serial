use mio_serial::{
    self,
    test::{self, SerialPortTestExt},
};

#[test]
fn test_stream_open() {
    let baud_rate = 9600;
    test::with_virtual_serial_ports::<_, std::convert::Infallible>(|port, _| {
        let builder = mio_serial::new(port, baud_rate);
        let stream = mio_serial::SerialStream::open(&builder)?;

        stream.expect_baud_rate(baud_rate)
    })
}
