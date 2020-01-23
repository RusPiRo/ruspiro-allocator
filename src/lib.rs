/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-allocator/0.4.0")]
#![cfg_attr(not(any(test, doctest)), no_std)]
#![feature(alloc_error_handler)]
//! # Custom Allocator for HEAP memory allocations
//!
//! This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
//! the ``alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
//! API to be used. It only encapsulates the memeory allocator that shall be linked into the binary.
//!
//! # Usage
//!
//! To link the custom allocator with your project just add the usage to your main crate rust file like so:
//! ```ignore
//! extern crate ruspiro_allocator;
//! ```
//! Wherever you define the usage of the ``ruspiro-allocator`` crate within your project does not matter. But as soon
//! as this is done the dynamic structures requiring heap memory allocations from the ``alloc`` crate could be used like
//! so:
//! ```
//! #[macro_use]
//! extern crate alloc;
//! use alloc::vec::*;
//! use alloc::boxed::*;
//!
//! fn main() {
//!     let mut v: Vec<u32> = vec![10, 20];
//!     let b: Box<u16> = Box::new(10);
//!     v.push(12);
//! }
//! ```
//!

/// this specifies the custom memory allocator to use whenever heap memory need to be allocated or freed
#[cfg_attr(not(any(test, doctest)), global_allocator)]
static ALLOCATOR: RusPiRoAllocator = RusPiRoAllocator;

use core::alloc::{GlobalAlloc, Layout};

mod memory;

struct RusPiRoAllocator;

unsafe impl GlobalAlloc for RusPiRoAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        memory::alloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        memory::free(ptr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = memory::alloc(layout.size(), layout.align());
        assert!((ptr as usize) < 0x3f00_0000);
        memset(ptr, 0x0, layout.size());
        ptr
    }
}

#[cfg(not(any(test, doctest)))]
#[alloc_error_handler]
#[allow(clippy::empty_loop)]
fn alloc_error_handler(_: Layout) -> ! {
    // TODO: how to handle memory allocation errors?
    loop {}
}

extern "C" {
    // reference to the compiler built-in function
    fn memset(ptr: *mut u8, value: i32, size: usize) -> *mut u8;
}
