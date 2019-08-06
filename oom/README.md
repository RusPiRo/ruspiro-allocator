# Custom Allocator OOM RusPiRo crate

The need for this crate is a bit weird and contains only the implementation of the ``rust_oom`` function that need to
be linked together with the custom allocator ``ruspiro-allocator``. This function can't be in the same crate as this leads
to the issue that the compiler complains that this function is already defined, however if not defining it in a separate crate
to link with the linker will complain with 'undefined reference to rust_oom'.

## Usage
This crate is a dependency from the ``ruspiro-allocator`` crate and should never used outside of it's parent.

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)