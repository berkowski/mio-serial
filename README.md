# mio-serial: A serial port IO library MIO.

[![Build Status](https://travis-ci.org/berkowski/mio-serial.svg?branch=master)](https://travis-ci.org/berkowski/mio-serial)
[![crates.io](http://meritbadge.herokuapp.com/mio-serial)](https://crates.io/crates/mio-serial)
[![docs.rs](https://docs.rs/mio-serial/badge.svg)](https://docs.rs/mio-serial)

mio-serial provides a serial port implementation using [mio](https://github.com/carllerche/mio).

## Usage

Add `mio-serial` to you `Cargo.toml`:

```toml
[dependencies]
mio-serial = "0.7"
```

Then add this to your crate root:

```rust
extern crate mio_serial;
```

## Examples
A few examples can be found [here](https://github.com/berkowski/mio-serial/tree/master/examples).

## License
This software is licensed under [MIT](https://opensource.org/licenses/MIT).

This software builds upon the [MPL-2.0](https://opensource.org/licenses/MPL-2.0) licensed [serialport-rs](https://gitlab.com/susurrus/serialport-rs) and 
constitutes a "Larger Work" by that license.  The source for [serialport-rs](https://gitlab.com/susurrus/serialport-rs) can be found at https://gitlab.com/susurrus/serialport-rs.
