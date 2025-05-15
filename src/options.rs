// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

pub struct FormatOptions {
    /// Do not edit the file but only check if it already applies this tools format.
    pub check: bool,
    /// Space(s) or tab(s) representing one level of indentation.
    pub indentation: String,
    // /// Whether to sort subjects, predicates and objects,
    // /// including within blank-nodes
    // pub sort_terms: bool,
    // /// Enables inserting new-lines before the following:
    // ///
    // /// - a subjects finalizing dot
    // /// - the first predicate of a subject
    // /// - the first object within one subject-predicate pair
    // /// - each objects within one subject-predicate pair
    // /// - each collection item;
    // ///   see <https://www.w3.org/TR/rdf12-turtle/#collections>
    // /// - each predicate within a blank-node
    // pub new_lines_for_easy_diff: bool,
    /// Whether to move a single/lone object
    /// (within one subject-predicate pair) onto a new line,
    /// or to keep it on the same line as the predicate.
    pub single_object_on_new_line: bool,
    /// Whether to cleanup/unify empty lines used as dividers.
    /// This ensures that there is exactly one empty line
    /// before and after each subject,
    /// and that there is none anywhere else.
    // pub cleanup_dividing_empty_lines: bool,
    /// Whether to force-write the output,
    /// even if potential issues with the formatting have been detected.
    pub force: bool,
    /// Sort blank nodes according to their `prtyr:sortingId` value.
    ///
    /// NOTE: For this to have an effect, [`Self::sort_terms`] needs to be enabled too.
    ///
    /// [`prtyr`](https://codeberg.org/elevont/prtyr)
    /// is an ontology concerned with
    /// [RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).
    pub prtyr_sorting: bool,
    /// Whether to use SPARQL-ish syntax for base and prefix,
    /// or the traditional Turtle syntax.
    ///
    /// - SPARQL-ish:
    ///
    ///   ```turtle
    ///   BASE <http://example.com/>
    ///   PREFIX foaf: <http://xmlns.com/foaf/0.1/>
    ///   ```
    ///
    /// - Traditional Turtle:
    ///
    ///   ```turtle
    ///   @base <http://example.com/> .
    ///   @prefix foaf: <http://xmlns.com/foaf/0.1/> .
    ///   ```
    pub sparql_syntax: bool,
    /// Whether maximize nesting of blank nodes,
    /// or rather use labels for all of them.
    ///
    /// NOTE That blank nodes referenced in more then one place can never be nested.
    pub max_nesting: bool,
    /// Whether to canonicalize the input before formatting.
    /// This refers to <https://www.w3.org/TR/rdf-canon/>,
    /// and effectively just label the blank nodes in a uniform way.
    pub canonicalize: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            check: true,
            indentation: "  ".to_string(),
            // sort_terms: false,
            // new_lines_for_easy_diff: false,
            single_object_on_new_line: false,
            force: false,
            prtyr_sorting: true,
            sparql_syntax: false,
            max_nesting: true,
            canonicalize: true,
        }
    }
}
