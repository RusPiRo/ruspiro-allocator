/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
//! Build script to pre-compile the heap memory allocation routines written in c for the time beeing
//! 
extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/memory.c")
        .compile("memory");

    cc::Build::new()
        .file("src/asm/memset.s")
        .compile("memset");
    
}