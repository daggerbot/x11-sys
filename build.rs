/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

extern crate bindgen;
extern crate pkg_config;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn generate_bindings() {
    let pkg_names = [
        "glx",
        "x11",
        "x11-xcb",
        "xcursor",
        "xext",
        "xfixes",
        "xft",
        "xi",
        "xinerama",
        "xmu",
        "xpresent",
        "xrandr",
        "xrender",
        "xss",
        "xt",
        "xtst",
        "xxf86vm",
    ];

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")
                                     .expect("missing CARGO_MANIFEST_DIR"));
    let lists_dir = manifest_dir.join("lists");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("missing OUT_DIR"));
    let header_path = out_dir.join("header.h");
    let bindings_path = out_dir.join("bindings.rs");

    let mut header_file = File::create(&header_path).expect("can't create header");
    let mut builder = bindgen::builder()
                      .allowlist_file(".*/GL/glx\\.h")
                      .allowlist_file(".*/GL/glxext\\.h")
                      .allowlist_file(".*/X11/\\.h");

    let mut any_libs_enabled = false;

    for pkg_name in pkg_names.into_iter() {
        let lib_feature = format!("CARGO_FEATURE_{}", pkg_name.to_uppercase().replace("-", "_"));
        let header_feature = format!("{}_HEADERS", lib_feature);

        // Handle pkg-config package name replacements.
        let pc_pkg_name = match pkg_name {
            "xss" => "xscrnsaver",
            _ => pkg_name,
        };

        if std::env::var_os(header_feature).is_some() {
            let pkg = pkg_config::probe_library(pc_pkg_name).expect("can't probe package");

            // Add include directories.
            for dir in pkg.include_paths {
                builder = builder.clang_arg(format!("-I{}", dir.display()));
            }

            // Add includes to our generated header.
            let header_list_path = lists_dir.join(format!("{}-headers.txt", pkg_name));
            let header_list_str = match std::fs::read_to_string(&header_list_path) {
                Ok(s) => s,
                Err(err) => panic!("{}: {}", header_list_path.display(), err),
            };
            for header in header_list_str.split_ascii_whitespace() {
                writeln!(header_file, "#include <{}>", header).expect("write failed");
            }
            println!("cargo:rerun-if-changed={}", header_list_path.display());

            if std::env::var_os(lib_feature).is_some() {
                any_libs_enabled = true;

                // Link libraries.
                for lib in pkg.libs {
                    println!("cargo:rustc-link-lib={}", lib);
                }

                // Add functions to allowlist.
                let fn_list_path = lists_dir.join(format!("{}-functions.txt", pkg_name));
                let fn_list_str = match std::fs::read_to_string(&fn_list_path) {
                    Ok(s) => s,
                    Err(err) => panic!("{}: {}", fn_list_path.display(), err),
                };
                for fn_name in fn_list_str.split_ascii_whitespace() {
                    builder = builder.allowlist_function(fn_name);
                }
                println!("cargo:rerun-if-changed={}", fn_list_path.display());

                // Add vars to allowlist.
                let var_list_path = lists_dir.join(format!("{}-vars.txt", pkg_name));
                let var_list_str = match std::fs::read_to_string(&var_list_path) {
                    Ok(s) => s,
                    Err(err) => panic!("{}: {}", var_list_path.display(), err),
                };
                for var_name in var_list_str.split_ascii_whitespace() {
                    builder = builder.allowlist_var(var_name);
                }
                println!("cargo:rerun-if-changed={}", var_list_path.display());
            }
        }
    }

    header_file.flush().expect("write failed");
    std::mem::drop(header_file);

    if !any_libs_enabled {
        builder = builder.blocklist_function(".*");
    }

    builder.header(header_path.display().to_string())
           .generate().expect("can't generate bindings")
           .write_to_file(&bindings_path).expect("write failed");
}

fn main() {
    if let Ok(target_os) = std::env::var("CARGO_CFG_TARGET_OS") {
        match target_os.as_str() {
            "dragonfly" | "freebsd" | "linux" | "netbsd" | "openbsd" => {
                generate_bindings();
            },
            "macos" => {
                if std::env::var_os("CARGO_FEATURE_MACOS").is_some() {
                    generate_bindings();
                }
            },
            _ => (),
        }
    }
}
