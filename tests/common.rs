#![allow(dead_code)]

#[cfg(unix)]
const DEFAULT_PORT_NAMES: &'static str = "/tty/USB0;/tty/USB1";
#[cfg(windows)]
const DEFAULT_PORT_NAMES: &'static str = "COM1;COM2";

pub struct TestOptions {
    pub port_names: Vec<String>,
}

pub fn setup() -> TestOptions {
    let port_names: Vec<String> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_PORT_NAMES)
        .split(';')
        .map(|s| s.to_owned())
        .collect();
    if port_names.len() < 2 {
        panic!("Expected two port names, found {}", port_names.len())
    }

    TestOptions { port_names }
}

#[derive(Debug)]
pub enum ValueError {
    BaudRate { expected: u32, actual: u32 },
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BaudRate { expected, actual } => write!(
                f,
                "BaudRate mismatch.  Expected {}.  Actual {}",
                expected, actual
            ),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Serial(mio_serial::Error),
    Value(ValueError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(ref e) => e.fmt(f),
            Self::Serial(ref e) => e.fmt(f),
            Self::Value(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<mio_serial::Error> for Error {
    fn from(error: mio_serial::Error) -> Self {
        Self::Serial(error)
    }
}

pub trait SerialPortTestExt: mio_serial::SerialPort {
    fn expect_baud_rate(&self, expected: u32) -> Result<(), Error> {
        let actual = self.baud_rate()?;

        if actual != expected {
            Err(Error::Value(ValueError::BaudRate { expected, actual }))
        } else {
            Ok(())
        }
    }
}

impl SerialPortTestExt for mio_serial::SerialStream {}

pub fn with_virtual_serial_ports<F>(test: F)
where
    F: FnOnce(&str, &str) -> Result<(), Error>,
{
    // Get port names from environment variable
    let port_names: Vec<String> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_PORT_NAMES)
        .split(';')
        .map(|s| s.to_owned())
        .collect();

    if port_names.len() < 2 {
        panic!("Expected two port names, found {}", port_names.len())
    }

    // run the test
    let result = test(port_names[0].as_str(), port_names[1].as_str());

    // Do any cleanup

    // Unwrap the result
    result.unwrap();
}
