# Change log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [4.0.0-beta2] 2021-07-16

### Added
- AsRawHandle, FromRawHandle, and IntoRawHandle impls for SerialStream on Windows

### Fixed
- Potential double Drop issue on Windows between NamedPipe and COMPort

## [4.0.0-beta1] 2021-07-13
This is a major update crossing two API-breaking dependency version jumps in `mio` and
`serialport-rs`.

### BREAKING CHANGES
This release contains multiple API breaking changes with the move to [serialport-rs](https://gitlab.com/sussurrrus/serialport-rs) v4.
Additional breaking changes were made to make the API more like mio/tokio where platform-specific
implimentation details are provided with `#cfg[]` guards instead of discrete structures like in `serialport-rs`

Specifically:

* Removed platform-specific `mio_serial::windows::Serial` and `mio_serial::unix::Serial`
* Added `mio_serial::SerialStream` with platform specific requirements at compile time with `#[cfg()]`
* Removed `COMPort::from_path`, use `SerialStream::open`
* Removed `TTYPort::from_path`, use `SerialStream::open`
* Removed `TTYPort::from_serial`.  Replaced with impl of `std::convert::TryFrom<serialport::TTYPort>`
* Removed `SerialPortSettings`, `serialport-rs` now uses the builder pattern

### Changed
* Removed "libudev" from the default features.  Still available for use when desired.
* Bumped [nix](https://github.com/nix-rust/nix) to 0.22
* Bumped [mio](https://github.com/tokio-rs/mio) to 0.7
* Bumped [serialport-rs](https://gitlab.com/sussurrrus/serialport-rs) to 4.0.0
* Changed CHANGELOG from asciidoc to markdown

### Added
* `SerialStream` structure as the common entry point for serial port IO.
* `SerialPortBuilderExt` extension trait to add `open_async` method
  to `serialport::SerialPortBuilder` much like the already existing `open` method.

### Other
* Switched CI to appveyor for Windows, OSX, and Linux.  It doesn't test as many targets, but some checks are better
  than none now that travis-ci is no longer an option.

## [3.3.1] 2020-03-15
### Added
* @flosse added #derive Debug support for the Serial struct in [#20](https://github.com/berkowski/mio-serial/pull/20)
* @vleesvlieg added automatic retrying for EINTR returns to file descriptors in [#21](https://github.com/berkowski/mio-serial/pull/21)

### Changed
* Bumped [nix](https://github.com/nix-rust/nix) to 0.17

## [3.3.0] 2019-08-23
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 3.3

## [3.2.14] 2019-06-01
### Changed
* Bumped [nix](https://github.com/nix-rust/nix) to 0.14 to address [#17](https://github.com/berkowski/mio-serial/issues/17)

## [3.2] 2019-01-12
### Changed
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 3.2

## [3.1.1] 2019-01-12
### Changed
* Merged [#16](https://github/berkowski/mio-serial/pull/16) @yuja fixed feature flags

## [3.1] 2018-11-10
### Added
* Added "libudev" feature.  Enabled by default, can be disabled for targets without udev support.

### Changed
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 3.1
* Merged [#13](https://github.com/berkowski/mio-serial/pull/13) @dvtomas added some clarity to the example.

## [3.0.1] - 2018-11-06
### Changed
* Restricted [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 3.0
  serialport-rs 3.1 contains API breaking changes.

## [3.0.0] - 2018-10-06
### Changed
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 3.0
  serialport-rs 3.0 contains breaking changes.
* Bumped [nix](https://github.com/nix-rust/nix) to 0.11
* `mio-serial` version number will now track upstream serialport-rs.  mio-serial
  is mostly feature complete at this point (at least for *nix) and this should
  help reduce confusion.

### Fixed
* Merged [#10](https://github.com/berkowski/mio-serial/pull/10) (thanks @yuja!).  Addresses some
  windows timeout settings.

## [0.8.0] - 2018-03-31
### Changed
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 2.3

### Added
* Merged[#5](https://github.com/berkowski/mio-serial/pull/5) @ndusart added `try_clone` implementations as requred
  by the serialport trait as of 2.3
* Closed[#6](https://github.com/berkowski/mio-serial/pull/6) @snorp also drew attention to the `try_clone` addition

## [0.7.0] - 2018-02-25
### Changed
* Bumped [serialport-rs](https://gitlab.com/susurrus/serialport-rs) to 2.1

### Added
* Merged[#4](https://github.com/berkowski/mio-serial/pull/4) @ndusart added windows support!
* Added appveyor config to support new windows impl.

## [0.6.0] - 2017-11-28
### Added
* Closed [#3](https://github.com/berkowski/mio-serial/pull/3) Reexport serialport::Error for error handling without importing serialport crate.
  Thanks @Idanko

## [0.5.0] - 2017-04-15
### Added
* Added [trust](https://github.com/japaric/trust) based ci

### Changed
* Changed license back to MIT now that `serialport-rs` is MPL-2.0
* Bumped `serialport-rs` dependency to 1.0

## [0.4.0] - 2017-02-13
### Changed
* Changed to LGPL-3 for compliance with `serialport` dependency.

## [0.3.0] - 2017-02-13 [YANKED]
### Added
* Bumped `serialport` dependency to 0.9
