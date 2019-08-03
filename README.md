# Custom Allocator RusPiRo crate

This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
the ``core::alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
API to be used. It only encapsulates the memeory allocator that shall be linked into the binary.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-allocator.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-allocator)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-allocator.svg)](https://crates.io/crates/ruspiro-allocator)
[![Documentation](https://docs.rs/ruspiro-allocator/badge.svg)](https://docs.rs/ruspiro-allocator)
[![License](https://img.shields.io/crates/l/ruspiro-allocator.svg)](https://github.com/RusPiRo/ruspiro-allocator#license)

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-allocator = "0.1.0"
```

Once done the access to the custom allocator is available and will be linked with your project if you add
the usage to your main crate rust file:
```
extern crate ruspiro_allocator;
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)