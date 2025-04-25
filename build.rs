// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::{env, fs};

fn main() {
    // We copy TreeSitter data to a subdirectory of the build directory
    let source_path = Path::new("tree-sitter");
    let build_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("tree-sitter");
    if !build_path.exists() {
        fs::create_dir(&build_path).unwrap();
    }
    fs::copy(
        source_path.join("grammar.js"),
        build_path.join("grammar.js"),
    )
    .unwrap();

    // We convert the TreeSitter grammar to C
    tree_sitter_generate::generate_parser_in_directory(&build_path, None, None, 14, None, None)
        .unwrap();

    // We build the C code
    let src_path = build_path.join("src");
    cc::Build::new()
        .include(&src_path)
        .file(src_path.join("parser.c"))
        .compile("parser");

    // We make sure the build is run again if the grammar changes
    println!(
        "cargo:rerun-if-changed={}",
        source_path.join("grammar.js").to_str().unwrap()
    );
}
