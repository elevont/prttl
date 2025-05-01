// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;
use tree_sitter::Node;

pub struct FormatOptions {
    /// Number of spaces used for one level of indentation
    pub indentation: usize,
    /// Whether to sort subjects, predicates and objects,
    /// including within blank-nodes
    pub sort_terms: bool,
    /// Enables inserting new-lines before the following:
    /// - a subjects finalizing dot
    /// - the first predicate of a subject
    /// - the first object within one subject-predicate pair
    /// - each objects within one subject-predicate pair
    /// - each collection item;
    ///   see <https://www.w3.org/TR/rdf12-turtle/#collections>
    /// - each predicate within a blank-node
    pub new_lines_for_easy_diff: bool,
    /// Whether to move a single/lone object
    /// (within one subject-predicate pair) onto a new line,
    /// or to keep it on the same line as the predicate.
    pub single_object_on_new_line: bool,
    /// Whether to cleanup/unify empty lines used as dividers.
    /// This ensures that there is exactly one empty line
    /// before and after each subject,
    /// and that there is none anywhere else.
    pub cleanup_dividing_empty_lines: bool,
    /// Whether to force-write the output,
    /// even if potential issues with the formatting have been detected.
    pub force: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indentation: 4,
            sort_terms: false,
            new_lines_for_easy_diff: false,
            single_object_on_new_line: false,
            cleanup_dividing_empty_lines: false,
            force: false,
        }
    }
}

impl FormatOptions {
    #[must_use]
    pub const fn includes_sorting(&self) -> bool {
        self.sort_terms
    }
}

/// Current state of the formatter.
#[derive(Default)]
pub struct Context<'tree> {
    /// The level of indentation
    /// (**not** measured in spaces).
    pub indentation_level: usize,
    pub subj: Option<Rc<Node<'tree>>>,
    pub pred: Option<Rc<Node<'tree>>>,
}
