/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
//! Build script to pre-compile the heap memory allocation routines written in c for the time beeing
//! 
extern crate cc;
use std::env;

fn main() {
    match env::var_os("CARGO_CFG_TARGET_ARCH") {
        Some(target_arch) => {
            if target_arch == "arm" {
                cc::Build::new()
                    .file("src/memory.c")
                    .compile("memory");

                cc::Build::new()
                    .file("src/asm/aarch32/memset.s")
                    .compile("memset");
            }

            if target_arch == "aarch64" {
                cc::Build::new()
                    .file("src/memory.c")
                    .define("AARCH64", None)
                    .compile("memory");
                
                cc::Build::new()
                    .file("src/asm/aarch64/memset.S")
                    .compile("memset");
            }
        }
        _ => ()
    }
}