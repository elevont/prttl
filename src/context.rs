// SPDX-FileCopyrightText: 2022 Helsing GmbH
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
