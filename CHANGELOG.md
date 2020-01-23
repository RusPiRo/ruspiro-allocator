# Changelog
## :pizza: v0.4.0
- ### :bulb: Features
    - Removed the whole ``C`` implementation and implement the allocator completely in ``Rust``.
    - Memory allocation is now completely lock free and uses atomic primitives to ensure cross core
    concurrent correctness when allocating and releasing memory