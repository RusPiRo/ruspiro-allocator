# RusPiRo - Custom Allocator

This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
the ``alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
API to be used. It only encapsulates the memory allocator that shall be linked into the binary.

![CI](https://github.com/RusPiRo/ruspiro-allocator/workflows/CI/badge.svg?branch=development)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-allocator.svg)](https://crates.io/crates/ruspiro-allocator)
[![Documentation](https://docs.rs/ruspiro-allocator/badge.svg)](https://docs.rs/ruspiro-allocator)
[![License](https://img.shields.io/crates/l/ruspiro-allocator.svg)](https://github.com/RusPiRo/ruspiro-allocator#license)

## Pre-Requisits

This crate requires to be buil with ``nightly`` as it uses the feature ``alloc_error_handler`` which is not stable yet.
When this crate is used with the Raspberry Pi it also requires the **MMU** to be configured and enables as it uses atomic operations to provide the lock free memory allocations.

## Usage

To use the crate just add the following dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-allocator = "||VERSION||"
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

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.
