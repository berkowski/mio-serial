use mio_serial::{self, test};
use std::fmt::Formatter;

#[derive(Debug)]
struct PortNotFound(String);

impl std::fmt::Display for PortNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find serial port named {}", self.0)
    }
}

impl std::error::Error for PortNotFound {}

/// Port enumeration doesn't seem to work on virtual serial ports created by com0com during
/// the CI process
#[test]
#[ignore = "Port enumeration test does not seem to work with com0com virtual ports"]
fn test_port_enumeration() {
    test::with_virtual_serial_ports(|port_a, port_b| {
        let names = [port_a, port_b];
        let ports = mio_serial::available_ports()?;
        for name in names {
            ports
                .iter()
                .find(|&info| info.port_name == name)
                .ok_or(test::Error::Other(PortNotFound(name.to_owned())))?;
        }
        Ok(())
    })
}
