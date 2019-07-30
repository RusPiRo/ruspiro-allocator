# Custom Allocator RusPiRo crate

This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
the ``core::alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
API to be used. It only encapsulates the memeory allocator that shall be linked into the binary.

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-allocator = "0.0.2"
```

Once done the access to the custom allocator is available and will be linked with your project if you add
the usage to your main crate rust file:
```
extern crate ruspiro_allocator;
```

## License
This crate is licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)