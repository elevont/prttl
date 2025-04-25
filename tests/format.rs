// SPDX-FileCopyrightText: 2023 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use pretty_assertions::assert_eq;
use turtlefmt::{format_turtle, FormatOptions};

fn fmt_opts_inverted() -> FormatOptions {
    FormatOptions {
        indentation: 2,
        sort_terms: true,
        new_lines_for_easy_diff: true,
        single_object_on_new_line: true,
        force: true,
    }
}

#[test]
fn test_format() {
    let input = include_str!("from.simple.ttl");
    let expected = include_str!("to.simple.ttl");
    assert_eq!(
        format_turtle(input, &FormatOptions::default()).unwrap(),
        expected
    );
}

#[test]
fn test_stable() {
    let file = include_str!("to.simple.ttl");
    assert_eq!(
        format_turtle(file, &FormatOptions::default()).unwrap(),
        file
    );
}

#[test]
fn test_format_default_inverted() {
    let input = include_str!("from.simple.ttl");
    let expected = include_str!("to.simple.default_inverted.ttl");
    let format_options = fmt_opts_inverted();
    assert_eq!(format_turtle(input, &format_options).unwrap(), expected);
}

#[test]
fn test_stable_default_inverted() {
    let file = include_str!("to.simple.default_inverted.ttl");
    let format_options = fmt_opts_inverted();
    assert_eq!(format_turtle(file, &format_options).unwrap(), file);
}
