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
