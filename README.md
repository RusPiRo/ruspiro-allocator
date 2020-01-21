# Custom Allocator RusPiRo crate

This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
the ``alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
API to be used. It only encapsulates the memory allocator that shall be linked into the binary.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-allocator.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-allocator)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-allocator.svg)](https://crates.io/crates/ruspiro-allocator)
[![Documentation](https://docs.rs/ruspiro-allocator/badge.svg)](https://docs.rs/ruspiro-allocator)
[![License](https://img.shields.io/crates/l/ruspiro-allocator.svg)](https://github.com/RusPiRo/ruspiro-allocator#license)

## Pre-Requisits

This crate requires to be buil with ``nightly`` as it uses the feature ``alloc_error_handler`` which is not stable yet.

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```toml
[dependencies]
ruspiro-allocator = "0.4"
```

Once done the access to the custom allocator is available and will be linked with your project if you add
the usage to your crates main rust file:
```rust
extern crate ruspiro_allocator;
```

Wherever you define the usage of the ``ruspiro-allocator`` crate the dynamic structures requiring heap memory allocations from the ``alloc`` crate could be used like so:
```rust
#[macro_use]
extern crate alloc;
use alloc::vec::*;
use alloc::boxed::*;

fn demo() {
    let mut v: Vec<u32> = vec![10, 20];
    let b: Box<u16> = Box::new(10);
    v.push(12);
}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
