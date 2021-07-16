mod common;
use mio_serial::available_ports;

#[test]
fn test_port_enumeration() {
    let options = common::setup();

    let ports = available_ports().expect("Unable to read available serial ports.");
    assert!(ports.len() >= 2);

    for name in options.port_names {
        ports.iter().find(|&info| info.port_name == name).expect(
            format!(
                "Unable to find serial port named {} in list of available ports",
                name
            )
            .as_str(),
        );
    }
}
