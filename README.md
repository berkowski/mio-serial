[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/mio-serial.svg
[crates-url]: https://crates.io/crates/mio-serial
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/berkowski/mio-serial/blob/master/LICENSE
[actions-badge]: https://github.com/berkowski/mio-serial/actions/workflows/github-ci.yml/badge.svg
[actions-url]: https://github.com/berkowski/mio-serial/actions?query=workflow%3Agithub-ci+branch%3Amaster

# mio-serial: A serial port IO library MIO.

mio-serial provides a serial port implementation using [mio](https://github.com/carllerche/mio).

## Usage

Add `mio-serial` to you `Cargo.toml`:

```toml
[dependencies]
mio-serial = "5.0.1"
```

Then add this to your crate root:

```rust
extern crate mio_serial;
```

## Features

The "libudev" dependency of `serialport-rs` is enabled by default.  For x86 linux systems this enables the `available_ports` function for port enumeration.
Not all targets support udev, especially when cross-compiling.  To disable this feature, compile with the `--no-default-features` option.  For example:

```
cargo build --no-default-features
```

### MSRV
The Minimum Supported Rust Version is **1.60.0** as found using [cargo-msrv](https://crates.io/crates/cargo-msrv)

## Examples
A few examples can be found [here](https://github.com/berkowski/mio-serial/tree/master/examples).

## Tests
Useful tests for serial ports require... serial ports, and serial ports are not often provided by online CI providers.
As so, automated build testing are really only check whether the code compiles, not whether it works.

Integration tests are in the `tests/` directory and typically require two serial ports to run.
The names of the serial ports can be configured at run time by setting the `TEST_PORT_NAMES` environment variable
to a semi-colon delimited string with the two serial port names.  The default values are:

- For Unix: `TEST_PORT_NAMES=/dev/ttyUSB0;/dev/ttyUSB1`
- For Windows: `TEST_PORT_NAMES=COM1;COM2`

**IMPORTANT** To prevent multiple tests from talking to the same ports at the same time make sure to limit the number
of test threads to 1 using:

```sh
cargo test -j1 -- --test-threads=1
```

## License
This software is licensed under [MIT](https://opensource.org/licenses/MIT).

This software builds upon the [MPL-2.0](https://opensource.org/licenses/MPL-2.0) licensed [serialport-rs](https://gitlab.com/susurrus/serialport-rs) and 
constitutes a "Larger Work" by that license.  The source for [serialport-rs](https://gitlab.com/susurrus/serialport-rs) can be found at https://gitlab.com/susurrus/serialport-rs.
