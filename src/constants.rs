// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

/// Base IRI used when parsing if no base is set,
/// and then again removed when formatting.
///
/// This ensures that relative IRIs in source files
/// that have no base set -
/// they require a base to be set when parsing -
/// still parse ok,
/// and that we still produce relative IRIs in our output again.
///
/// The actual value of this is not very important.
/// It just has to be a valid IRI,
/// should have a '/' or '#' at the end,
/// and should be so obscure that we will not likely encounter it
/// in any real-world input.
pub const SUBSTITUTE_BASE: &str = "http://a1234567890.substitute.base/";
