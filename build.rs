//! Build script to pre-compile the heap memory allocation routines written in c for the time beeing
//! 
extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/memory.c")
        .compile("memory");
    
}