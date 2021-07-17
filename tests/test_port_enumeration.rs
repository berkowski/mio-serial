mod common;
use mio_serial::available_ports;

/// Port enumeration doesn't seem to work on virtual serial ports created by com0com during
/// the CI process
#[test]
#[ignore]
fn test_port_enumeration() {
    let options = common::setup();

    let ports = available_ports().expect("Unable to read available serial ports.");

    for name in options.port_names {
        ports.iter().find(|&info| info.port_name == name).expect(
            format!(
                "Unable to find serial port named {} in list of available ports ({:?})",
                name,
                ports.clone()
            )
            .as_str(),
        );
    }
}
