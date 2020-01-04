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
    if let Some(target_arch) = env::var_os("CARGO_CFG_TARGET_ARCH") {
        if target_arch == "arm" {
            cc::Build::new().file("src/memory.c").compile("memory");
        }

        if target_arch == "aarch64" {
            cc::Build::new()
                .file("src/memory.c")
                .define("AARCH64", None)
                .compile("memory");
        }
    }
}
