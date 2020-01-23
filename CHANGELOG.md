# Changelog
## :pizza: v0.4.0
- ### :bulb: Features
    - Removed the whole ``C`` implementation and implement the allocator completely in ``Rust``.
    - Memory allocation is now completely lock free and uses atomic primitives to ensure cross core
    concurrent correctness when allocating and releasing memory

## :carrot: v0.3.1 Release Notes
- ### 🔧 Maintenance
    - Remove usage of memset assembly
    - Remove use of custom oom handler trick as the #[alloc_error_handler] seem now implemented in Rust correctly.