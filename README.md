# mio-serial: A serial port IO library MIO.

[![Build status](https://ci.appveyor.com/api/projects/status/1j0fy1f5k7h14x95/branch/master?svg=true)](https://ci.appveyor.com/project/berkowski/mio-serial/branch/master)
[![crates.io](http://shields.io/crates/v/mio-serial)](https://crates.io/crates/mio-serial)
[![docs.rs](https://docs.rs/mio-serial/badge.svg)](https://docs.rs/mio-serial)

mio-serial provides a serial port implementation using [mio](https://github.com/carllerche/mio).

## NOTICE
This crate is no longer actively maintained (see [#25](https://github.com/berkowski/mio-serial/issues/25)) and is
open for adoption.  Create an issue if interested.

## Usage

Add `mio-serial` to you `Cargo.toml`:

```toml
[dependencies]
mio-serial = "4"
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

## Examples
A few examples can be found [here](https://github.com/berkowski/mio-serial/tree/master/examples).

## License
This software is licensed under [MIT](https://opensource.org/licenses/MIT).

This software builds upon the [MPL-2.0](https://opensource.org/licenses/MPL-2.0) licensed [serialport-rs](https://gitlab.com/susurrus/serialport-rs) and 
constitutes a "Larger Work" by that license.  The source for [serialport-rs](https://gitlab.com/susurrus/serialport-rs) can be found at https://gitlab.com/susurrus/serialport-rs.
