use mio_serial::{
    self,
    test::{self, SerialPortTestExt},
    SerialPortBuilderExt,
};

#[test]
fn test_builder_open_async() {
    // let options = common::setup();
    // let port_name = options.port_names[0].clone();
    test::with_virtual_serial_ports::<_, std::convert::Infallible>(|port, _| {
        let baud_rate = 9600;
        let builder = mio_serial::new(port, baud_rate);

        let stream = builder.open_async()?;

        stream.expect_baud_rate(baud_rate)
    })
}
