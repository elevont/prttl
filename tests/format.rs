// SPDX-FileCopyrightText: 2023 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

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
        canonicalize: true,
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

#[test]
fn test_format() -> Result<(), Error> {
    let input = include_str!("data/input/pretty_printing/simple.ttl");
    let expected = include_str!("data/output/pretty_printing/simple.ttl");
    assert_eq!(format_turtle(input, FormatOptions::default())?, expected);
    Ok(())
}

#[test]
fn test_stable() -> Result<(), Error> {
    let file = include_str!("data/output/pretty_printing/simple.ttl");
    assert_eq!(format_turtle(file, FormatOptions::default())?, file);
    Ok(())
}

#[test]
fn test_format_strict() -> Result<(), Error> {
    let input = include_str!("data/input/pretty_printing/simple.ttl");
    let expected = include_str!("data/output/pretty_printing/simple.strict.ttl");
    assert_eq!(format_turtle(input, fmt_opts_strict(true))?, expected);
    Ok(())
}

#[test]
fn test_stable_default_inverted() -> Result<(), Error> {
    let file = include_str!("data/output/pretty_printing/simple.strict.ttl");
    assert_eq!(format_turtle(file, fmt_opts_strict(true))?, file);
    Ok(())
}

#[test]
fn test_blank_nodes_prtr() -> Result<(), Error> {
    let input = include_str!("data/input/pretty_printing/blank_nodes_prtr.ttl");
    let expected = include_str!("data/output/pretty_printing/blank_nodes_prtr.ttl");
    // assert_eq!(format_turtle(input, FormatOptions::default())?, expected);
    assert_eq!(format_turtle(input, fmt_opts_strict(false))?, expected);
    Ok(())
}
