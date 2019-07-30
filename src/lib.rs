/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-allocator/0.0.2")]
#![no_std]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
//! # Custom Allocator for HEAP memory allocations
//! 
//! This crate provides a custom allocator for heap memory. If any baremetal crate uses functions and structures from
//! the ``core::alloc`` crate an allocator need to be provided as well. However, this crate does not export any public
//! API to be used. It only encapsulates the memeory allocator that shall be linked into the binary.
//! 
//! # Usage
//! 
//! To link the custom allocator with your project just add the usage to your main crate rust file like so:
//! ```
//! extern crate ruspiro_allocator;
//! ```
//! 

/// this specifies the custom memory allocator to use whenever heap memory need to be allocated or freed
#[global_allocator]
static ALLOCATOR: RusPiRoAllocator = RusPiRoAllocator;

use core::alloc::{GlobalAlloc, Layout};

struct RusPiRoAllocator;

unsafe impl GlobalAlloc for RusPiRoAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        m_alloca(layout.size() as u32, layout.align() as u16)        
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        m_freea(ptr)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = m_alloca(layout.size() as u32, layout.align() as u16);
        m_memset(ptr, 0x0, layout.size() as u32);
        ptr        
    }
}


#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
    // TODO: how to handle memory allocation errors?
    loop { }
}

extern "C" {
    fn m_alloca(size: u32, align: u16) -> *mut u8;
    fn m_freea(ptr: *mut u8);
    fn m_memset(ptr: *mut u8, value: u32, size: u32);
}

// including the assembly file containing fast memset function
global_asm!(include_str!("./asm/memset.s"));