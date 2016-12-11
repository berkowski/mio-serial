# mio-serial: A termios serial implementation for mio

[![crates.io](http://meritbadge.herokuapp.com/mio-serial)](https://crates.io/crates/mio-serial)
[![docs.rs](https://docs.rs/mio-serial/badge.svg)](https://docs.rs/mio-serial)

mio-serial provides a termios serial port implementation for
[mio](https://github.com/carllerche/mio).  As this uses termios
there is **no** windows implementation at this time.

## Usage

Add `mio-serial` to you `Cargo.toml`:

```toml
[dependencies]
mio-serial = "0.1"
```

Then add this to your crate root:

```rust
extern crate mio_serial;
```

## Examples
A few examples can be found [here](https://github.com/berkowski/mio-serial/tree/master/examples).
