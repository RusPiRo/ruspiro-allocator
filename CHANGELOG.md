# Changelog

## :peach: v0.4.3

This version migtrates to GitHub Actions as new CI/CD pipeline. In addition it introduces a custom build target used to build the crate for the Raspberry Pi Aarch64.

## :peach: v0.4.2

This version introduces a stable build pipeline using Travis-CI. This provides a convinient way to publish next crate versions from the pipeline.

- ### :wrench: Maintenance

  - Adjusted the file headers to reflect copyright as of 2020 and the correct author
  - add the travis-CI configuration

## :banana: v0.4.1

- ### :detective: Bug-Fixes

  - Issue [8](https://github.com/RusPiRo/ruspiro-allocator/issues/8) : If previously freed memory was re-used for a new allocation that did not fit into a fixed memory bucket could lead to memory curruption on the heap if the re-uses size was larger than the original size of the re-used block.

- ### :wrench: Maintenance

  - Switch to `cargo-make` and `Makefile.toml` to run the reliable build process, locally and in CI.

## :pizza: v0.4.0

- ### :bulb: Features

  - Removed the whole ``C`` implementation and implement the allocator completely in ``Rust``.
  - Memory allocation is now completely lock free and uses atomic primitives to ensure cross core
    concurrent correctness when allocating and releasing memory

## :carrot: v0.3.1 Release Notes

- ### :wrench: Maintenance

  - Remove usage of memset assembly
  - Remove use of custom oom handler trick as the #[alloc_error_handler] seem now implemented in Rust correctly.