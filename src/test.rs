//! Reusable test fixtures for async code development
//!
//! These traits and types are intended to help test `mio-serial` and
//! the closely related `tokio-serial` code bases with minimal copying
//! between the two code bases.
#![allow(dead_code)]

#[cfg(unix)]
const DEFAULT_PORT_NAMES: &'static str = "/tty/USB0;/tty/USB1";
#[cfg(windows)]
const DEFAULT_PORT_NAMES: &'static str = "COM1;COM2";

/// A serial port config mismatch error
///
/// Raised for tests attempting to verify serial port parameters
#[derive(Debug)]
pub enum ValueError {
    /// Baudrate mismatch
    BaudRate {
        /// Expected Baudrate
        expected: u32,
        /// Actual Baudrate
        actual: u32,
    },
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

/// Integration test error types
#[derive(Debug)]
pub enum Error<T>
where
    T: std::error::Error,
{
    /// An I/O error raised outside of the serialport interface
    Io(std::io::Error),
    /// A mio_serial::Error raised within the serialport interface
    Serial(crate::Error),
    /// A port configuration mismatch
    Value(ValueError),
    /// User defined other type
    Other(T),
}

impl<T> std::fmt::Display for Error<T>
where
    T: std::error::Error,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(ref e) => e.fmt(f),
            Self::Serial(ref e) => e.fmt(f),
            Self::Value(ref e) => e.fmt(f),
            Self::Other(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl<T> std::error::Error for Error<T> where T: std::error::Error {}

impl<T> From<std::io::Error> for Error<T>
where
    T: std::error::Error,
{
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl<T> From<crate::Error> for Error<T>
where
    T: std::error::Error,
{
    fn from(error: crate::Error) -> Self {
        Self::Serial(error)
    }
}

/// An extension trait that adds methods to help test serial port parameters
pub trait SerialPortTestExt<T>: crate::SerialPort
where
    T: std::error::Error,
{
    /// Test an expected baud rate value
    fn expect_baud_rate(&self, expected: u32) -> Result<(), Error<T>> {
        let actual = self.baud_rate()?;

        if actual != expected {
            Err(Error::Value(ValueError::BaudRate { expected, actual }))
        } else {
            Ok(())
        }
    }
}

impl<T> SerialPortTestExt<T> for crate::SerialStream where T: std::error::Error {}

/// Generic fixture for testing serialports with real or virtual hardware
///
/// The user should provide three closures:
///
/// - `setup(&str, &str) -> R` is provided two names of the intended serial ports to test.
///   The port names are derived from the `TEST_PORT_NAMES` environment variable if present,
///   the default values.  This can be used to run arbitrary code to get the intended serial
///   ports ready
///
/// - `test(&str, &str) -> Result<(), Error<T>>` This is your actual integration test
///
/// - `teardown(R) -> ()` This is your time to clean up after code run during the setup phase
///
///
/// Often you'll get an error about type inference of the 'Other' error type:
///
/// ```text
/// error[E0282]: type annotations needed
///   --> tests\test_open.rs:10:5
///    |
/// 10 |     test::with_virtual_serial_ports(|port, _| {
///    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type for type parameter `T` declared on the function `with_virtual_serial_ports`
/// ...
/// 14 |         stream.expect_baud_rate(baud_rate)
///    |         ---------------------------------- this method call resolves to `Result<(), mio_serial::test::Error<T>>`
///```
///
/// If you know you're not going to use the Other type a quick way to solve this is to use
/// `std::convert::Infallable` like so:
///
/// ```text
/// test::with_virtual_serial_ports::<_, std::convert::Infallible>(|port, _| { ...
/// ```
pub fn with_virtual_serial_ports_setup_fixture<S, F, T, C, R>(setup: S, test: F, teardown: C)
where
    T: std::error::Error,
    F: FnOnce(&str, &str) -> Result<(), Error<T>>,
    S: FnOnce(&str, &str) -> R,
    C: FnOnce(R),
{
    let port_names: Vec<String> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_PORT_NAMES)
        .split(';')
        .map(|s| s.to_owned())
        .collect();

    if port_names.len() < 2 {
        panic!("Expected two port names, found {}", port_names.len())
    }

    let port_a = port_names[0].as_str();
    let port_b = port_names[1].as_str();

    let fixture = setup(port_a, port_b);
    let result = test(port_a, port_b);
    teardown(fixture);
    result.unwrap();
}

/// Convenience fixture for testing serialports without any setup or teardown
///
/// Calls `with_virtual_serial_ports_setup_fixtures` with empty `setup` and `teardown`
/// fixtures.  Requires the desired serial ports to already be ready for use.
///
/// Often you'll get an error about type inference of the 'Other' error type:
///
/// ```text
/// error[E0282]: type annotations needed
///   --> tests\test_open.rs:10:5
///    |
/// 10 |     test::with_virtual_serial_ports(|port, _| {
///    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type for type parameter `T` declared on the function `with_virtual_serial_ports`
/// ...
/// 14 |         stream.expect_baud_rate(baud_rate)
///    |         ---------------------------------- this method call resolves to `Result<(), mio_serial::test::Error<T>>`
///```
///
/// If you know you're not going to use the Other type a quick way to solve this is to use
/// `std::convert::Infallable` like so:
///
/// ```text
/// test::with_virtual_serial_ports::<_, std::convert::Infallible>(|port, _| { ...
/// ```
pub fn with_virtual_serial_ports<F, T>(test: F)
where
    T: std::error::Error,
    F: FnOnce(&str, &str) -> Result<(), Error<T>>,
{
    with_virtual_serial_ports_setup_fixture(|_, _| {}, test, |_| {});
    // // Get port names from environment variable
    // let port_names: Vec<String> = std::option_env!("TEST_PORT_NAMES")
    //     .unwrap_or(DEFAULT_PORT_NAMES)
    //     .split(';')
    //     .map(|s| s.to_owned())
    //     .collect();

    // if port_names.len() < 2 {
    //     panic!("Expected two port names, found {}", port_names.len())
    // }

    // // run the test
    // let result = test(port_names[0].as_str(), port_names[1].as_str());

    // // Do any cleanup

    // // Unwrap the result
    // result.unwrap();
}
