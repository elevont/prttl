// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

pub struct FormatOptions {
    /// Do not edit the file but only check if it already applies this tools format.
    pub check: bool,
    /// Space(s) or tab(s) representing one level of indentation.
    pub indentation: String,
    /// Whether to move a single/lone object
    /// (within one subject-predicate pair) onto a new line,
    /// or to keep it on the same line as the predicate.
    pub single_leafed_new_lines: bool,
    /// Whether to force-write the output,
    /// even if potential issues with the formatting have been detected.
    ///
    /// One such issue would be,
    /// if comments have been found in the input.
    /// Because they will be completely removed in the output,
    /// we require `force = true` to try to avoid unintentional loss of information.
    pub force: bool,
    /// Sort blank nodes according to their `prtr:sortingId` value.
    ///
    /// [`prtr`](https://codeberg.org/elevont/prtr)
    /// is an ontology concerned with
    /// [RDF Pretty Printing](https://www.w3.org/DesignIssues/Pretty.html).
    pub prtr_sorting: bool,
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
    /// Warn if a double or decimal literal can not be formatted as native Turtle literal.
    ///
    /// Turtles DOUBLE supports less formats then `xsd:double`,
    /// and DECIMAL supports less formats then `xsd:decimal`.
    /// See <https://github.com/w3c/rdf-turtle/issues/98> for more details.
    pub warn_unsupported_numbers: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            check: true,
            indentation: "  ".to_string(),
            single_leafed_new_lines: false,
            force: false,
            prtr_sorting: true,
            sparql_syntax: false,
            max_nesting: true,
            canonicalize: true,
            warn_unsupported_numbers: true,
        }
    }
}
