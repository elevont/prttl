// SPDX-FileCopyrightText: 2023 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::Path, rc::Rc};

#[cfg(test)]
use pretty_assertions::assert_eq;
use prttl::{error::Error, formatter::format, options::FormatOptions, parser};

fn fmt_opts_strict(single_object_on_new_line: bool) -> FormatOptions {
    FormatOptions {
        indentation: "  ".to_string(),
        single_leafed_new_lines: single_object_on_new_line,
        force: true,
        prtr_sorting: true,
        check: false,
        sparql_syntax: false,
        max_nesting: true,
        canonicalize: false,
        warn_unsupported_numbers: true,
        subject_type_order_preset: None,
        subject_type_order: None,
        predicate_order_preset: None,
        predicate_order: None,
    }
}

fn format_turtle(original: &str, options: FormatOptions) -> Result<String, Error> {
    let options = Rc::new(options);
    let input = parser::parse(original.as_bytes(), &options)?;
    format(&input, options)
}

fn test_format(
    input: &str,
    expected: &str,
    debug_file: &Path,
    expected_file: &Path,
    fmt_options: FormatOptions,
) -> Result<(), Error> {
    let output = format_turtle(input, fmt_options)?;
    let debug_file_abs = std::path::absolute(format!("target/tests/{}", debug_file.display())).unwrap();
    std::fs::create_dir_all(debug_file_abs.parent().unwrap()).unwrap();
    if output != expected {
        std::fs::write(debug_file_abs, &output).unwrap();
        eprintln!(
            "Debug out file written to:\n{}\n\nCompare with:\nmeld tests/{} target/tests/{} &",
            debug_file.display(),
            expected_file.display(),
            debug_file.display()
        );
    } else if fs::exists(&debug_file_abs).unwrap() {
        std::fs::remove_file(debug_file_abs).unwrap();
    }
    assert_eq!(output, expected);
    Ok(())
}

macro_rules! test_auto {
    ($input:literal, $expected:literal, $stable:literal, $strict:literal, $single_object_on_new_line:literal) => {
        test_format(
            include_str!($input),
            include_str!($expected),
            Path::new(&format!(
                "{}{}{}{}.actual_output.ttl",
                $expected,
                if $stable { ".stable" } else { "" },
                if $strict { ".strict" } else { "" },
                if $single_object_on_new_line {
                    ".single_object_on_new_line"
                } else {
                    ""
                }
            )),
            Path::new($expected),
            if $strict {
                fmt_opts_strict($single_object_on_new_line)
            } else {
                let mut fmt_options = FormatOptions::default();
                if $single_object_on_new_line {
                    fmt_options.single_leafed_new_lines = true;
                }
                fmt_options.canonicalize = false;
                fmt_options
            },
        )
    };
    ($input:literal, $expected:literal, $strict:literal, $single_object_on_new_line:literal) => {
        test_auto!(
            $input,
            $expected,
            false,
            $strict,
            $single_object_on_new_line
        )
    };
    ($input:literal, $strict:literal, $single_object_on_new_line:literal) => {
        test_auto!($input, $input, true, $strict, $single_object_on_new_line)
    };
}

#[test]
fn test_simple() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/simple.ttl",
        "data/output/pretty_printing/simple.ttl",
        false,
        false
    )
}

#[test]
fn test_simple_stable() -> Result<(), Error> {
    test_auto!("data/output/pretty_printing/simple.ttl", false, false)
}

#[test]
fn test_simple_strict() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/simple.ttl",
        "data/output/pretty_printing/simple.ttl",
        true,
        true
    )
}

#[test]
fn test_simple_strict_stable() -> Result<(), Error> {
    test_auto!("data/output/pretty_printing/simple.ttl", true, true)
}

#[test]
fn test_blank_nodes_prtr_strict() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/blank_nodes_prtr.ttl",
        "data/output/pretty_printing/blank_nodes_prtr.ttl",
        true,
        false
    )
}

#[test]
fn test_blank_nodes_prtr_strict_stable() -> Result<(), Error> {
    test_auto!(
        "data/output/pretty_printing/blank_nodes_prtr.ttl",
        true,
        false
    )
}

#[test]
fn test_tbl_diff() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/tbl-diff.ttl",
        "data/output/pretty_printing/tbl-diff.ttl",
        true,
        false
    )
}

#[test]
fn test_all() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/all.ttl",
        "data/output/pretty_printing/all.ttl",
        true,
        false
    )
}

// #[test]
// fn test_all_stable() -> Result<(), Error> {
//     test_auto!(
//         "data/output/pretty_printing/all.ttl",
//         true,
//         false
//     )
// }

// #[test]
// fn test_all_strict() -> Result<(), Error> {
//     test_auto!(
//         "data/input/pretty_printing/all.ttl",
//         "data/output/pretty_printing/all.strict.ttl",
//         true,
//         true
//     )
// }

// #[test]
// fn test_all_strict_stable() -> Result<(), Error> {
//     test_auto!(
//         "data/output/pretty_printing/all.strict.ttl",
//         true,
//         true
//     )
// }

#[test]
fn test_all_prtr() -> Result<(), Error> {
    test_auto!(
        "data/input/pretty_printing/all_prtr.ttl",
        "data/output/pretty_printing/all_prtr.ttl",
        true,
        false
    )
}

#[test]
fn test_all_prtr_stable() -> Result<(), Error> {
    test_auto!("data/output/pretty_printing/all_prtr.ttl", true, false)
}
