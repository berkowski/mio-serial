mod common;
use mio_serial::SerialPortBuilderExt;

#[test]
fn test_builder_open_async() {
    // let options = common::setup();
    // let port_name = options.port_names[0].clone();
    common::with_serial_ports::<_, std::convert::Infallible>(|port, _| {
        let baud_rate = 9600;
        let builder = mio_serial::new(port, baud_rate);

        let stream = builder.open_async()?;

        async_serial_test_helper::expect_baud_rate(&stream, baud_rate)
    })
}
