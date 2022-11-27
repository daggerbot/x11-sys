# x11-sys

This Rust crate provides bindings to the X11 libraries.
This is similar to the `x11` crate but uses `bindgen` to generate bindings rather than having them written by hand.
`x11-sys` cannot replace `x11-dl`, as `bindgen` currently does not support generating library structs.
