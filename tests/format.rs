// // SPDX-FileCopyrightText: 2023 Helsing GmbH
// //
// // SPDX-License-Identifier: Apache-2.0

// #[cfg(test)]
// use pretty_assertions::assert_eq;
// use turtlefmt::{format_turtle, options::FormatOptions};

// type BoxResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

// fn fmt_opts_inverted() -> FormatOptions {
//     FormatOptions {
//         indentation: 2,
//         sort_terms: true,
//         new_lines_for_easy_diff: true,
//         single_object_on_new_line: true,
//         cleanup_dividing_empty_lines: true,
//         force: true,
//         prtyr_sorting: false,
//     }
// }

// fn fmt_diff_human_mix_opt() -> FormatOptions {
//     FormatOptions {
//         indentation: 2,
//         sort_terms: true,
//         new_lines_for_easy_diff: true,
//         single_object_on_new_line: false,
//         cleanup_dividing_empty_lines: true,
//         force: true,
//         prtyr_sorting: true,
//     }
// }

// #[test]
// fn test_format() {
//     let input = include_str!("from.simple.ttl");
//     let expected = include_str!("to.simple.ttl");
//     assert_eq!(
//         format_turtle(input, &FormatOptions::default()).unwrap(),
//         expected
//     );
// }

// #[test]
// fn test_stable() {
//     let file = include_str!("to.simple.ttl");
//     assert_eq!(
//         format_turtle(file, &FormatOptions::default()).unwrap(),
//         file
//     );
// }

// #[test]
// fn test_format_default_inverted() {
//     let input = include_str!("from.simple.ttl");
//     let expected = include_str!("to.simple.default_inverted.ttl");
//     let format_options = fmt_opts_inverted();
//     assert_eq!(format_turtle(input, &format_options).unwrap(), expected);
// }

// #[test]
// fn test_stable_default_inverted() {
//     let file = include_str!("to.simple.default_inverted.ttl");
//     let format_options = fmt_opts_inverted();
//     assert_eq!(format_turtle(file, &format_options).unwrap(), file);
// }

// #[test]
// fn test_blank_nodes_prtyr() -> BoxResult<()> {
//     let input = include_str!("from.blank_nodes_prtyr.ttl");
//     // let input = include_str!("from.simple.ttl");
//     let expected = include_str!("to.blank_nodes_prtyr.ttl");
//     let format_options = fmt_diff_human_mix_opt();
//     assert_eq!(format_turtle(input, &format_options)?, expected);
//     Ok(())
// }
