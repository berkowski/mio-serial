# mio-serial: A serial port IO library MIO.

[![crates.io](http://meritbadge.herokuapp.com/mio-serial)](https://crates.io/crates/mio-serial)
[![docs.rs](https://docs.rs/mio-serial/badge.svg)](https://docs.rs/mio-serial)

mio-serial provides a serial port implementation for [mio](https://github.com/carllerche/mio).  

**Note:** At the moment this is unix (termios) only.  No windows COM port yet.

## Usage

Add `mio-serial` to you `Cargo.toml`:

```toml
[dependencies]
mio-serial = "0.2"
```

Then add this to your crate root:

```rust
extern crate mio_serial;
```

## Examples
A few examples can be found [here](https://github.com/berkowski/mio-serial/tree/master/examples).
