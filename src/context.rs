// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Write;

/// Current state of the formatter.
#[derive(Default)]
pub struct Context<W: Write> {
    /// The level of indentation
    /// (**not** measured in spaces).
    pub indent_level: usize,
    pub output: W,
}
