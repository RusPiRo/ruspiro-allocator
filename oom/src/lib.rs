/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-allocator-oom/0.0.1")]
#![no_std]

//! # Allocator OutOfMemory implementation
//! 

// putting this into the same crate that defines the alloc_error_handler lead to "rust_oom"
// already defined during compile time, but leaving this out there lead to "undefined reference to rust_oom" 
// .... WTF ...
// so defining this in this crate that does not define the alloc_error_handler and link it to the allocator crate
// -> this wotks fine....
#[no_mangle]
fn rust_oom() -> ! {
    // well, currently there is nothing we could do on out-of-memory other than halt the core
    loop { }
}
