// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Error, Result};
use grammar::{
    is_turtle_decimal, is_turtle_double, is_turtle_integer, NodeKind, RootContext, StringDecoder,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use tree_sitter::{Language, Node};

mod grammar;

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

fn get_tree_sitter_turtle() -> Language {
    extern "C" {
        fn tree_sitter_turtle() -> Language;
    }
    unsafe { tree_sitter_turtle() }
}

fn format_turtle_once(original: &str, options: &FormatOptions) -> Result<String> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&get_tree_sitter_turtle())?;
    let tree = parser.parse(original.as_bytes(), None).unwrap();

    let mut formatted = String::new();
    TurtleFormatter {
        file: original.as_bytes(),
        output: &mut formatted,
        options,
        prefixes: HashMap::new(),
        seen_comments: false,
    }
    .fmt_doc(tree.root_node())?;
    Ok(formatted)
}

pub fn format_turtle(original: &str, options: &FormatOptions) -> Result<String> {
    let mut result = format_turtle_once(original, options)?;
    if options.includes_sorting() {
        // This is necessary because the sorting of potentially reformatted terms
        // (e.g. 'bar' -> "bar") might change sort order.
        result = format_turtle_once(&result, options)?;
    }
    Ok(result)
}

struct TurtleFormatter<'a, W: Write> {
    file: &'a [u8],
    output: W,
    options: &'a FormatOptions,
    prefixes: HashMap<String, String>,
    seen_comments: bool,
}

impl<W: Write> TurtleFormatter<'_, W> {
    fn fmt_doc(&mut self, node: Node<'_>) -> Result<()> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::TurtleDoc);
        let mut context = RootContext::Start;
        let mut row = node.start_position().row;
        let mut prefix_buffer: Vec<(Node<'_>, Vec<Node<'_>>)> = Vec::new();

        let children = self.iter_children_sorted(
            node,
            self.options.sort_terms,
            |n| NodeKind::from(&n) == NodeKind::Triples,
            |n| {
                for sn in n.children_by_field_name("subject", &mut n.walk()) {
                    let sn_cont = sn.utf8_text(self.file).unwrap_or("");
                    if sn_cont == "<" || sn_cont == ">" {
                        continue;
                    }
                    return Some(sn);
                }
                None
            },
        )?;
        for child in children {
            match NodeKind::from(&child) {
                NodeKind::Comment => {
                    if child.start_position().row == row {
                        if let Some((_, prefix_comments)) = prefix_buffer.last_mut() {
                            // We keep the comment connected to the prefixes
                            prefix_comments.push(child);
                        } else {
                            // Inline comment
                            self.fmt_comments([child], true)?;
                            if context == RootContext::Start {
                                context = RootContext::Comment;
                            }
                        }
                    } else {
                        // Block comment
                        self.fmt_possible_prefixes(&mut prefix_buffer, &mut context)?;
                        if context != RootContext::Start {
                            for _ in 0..(child.start_position().row - row).clamp(
                                if context == RootContext::Comment {
                                    1
                                } else {
                                    2
                                },
                                4,
                            ) {
                                writeln!(self.output)?;
                            }
                        }
                        self.fmt_comments([child], false)?;
                        context = RootContext::Comment;
                    }
                }
                NodeKind::Base => {
                    self.fmt_possible_prefixes(&mut prefix_buffer, &mut context)?;
                    if context != RootContext::Start {
                        writeln!(self.output)?;
                    }
                    if context == RootContext::Triples {
                        writeln!(self.output)?;
                    }
                    context = RootContext::Prefixes;
                    self.fmt_base(child)?;
                }
                NodeKind::Prefix => {
                    prefix_buffer.push((child, Vec::new()));
                }
                NodeKind::Triples => {
                    self.fmt_possible_prefixes(&mut prefix_buffer, &mut context)?;
                    if context != RootContext::Start {
                        if context != RootContext::Comment || child.start_position().row > row + 1 {
                            writeln!(self.output)?;
                        }
                        writeln!(self.output)?;
                    }
                    self.fmt_triples(child)?;
                    context = RootContext::Triples;
                }
                _ => bail!("Unexpected turtle_doc child: {}", child.to_sexp()),
            }
            row = child.end_position().row;
        }
        self.fmt_possible_prefixes(&mut prefix_buffer, &mut context)?;
        writeln!(self.output)?;
        if self.options.includes_sorting() && self.seen_comments {
            eprintln!(
                "WARNING: You have chosen to sort terms \
while your source contains comments. \
This is not properly supported by this tool, \
so your comments will almost certainly end-up in the wrong place."
            );
            if self.options.force {
                eprintln!(
                    "WARNING: ... as you have chosen to force write (--force), \
the formatting result has been written to file anyway."
                );
            } else {
                eprintln!(
                    "ERROR: ... as you have not chosen to force write (--force), \
the formatting result has not been written to file."
                );
                bail!(
                    "Not allowed to sort terms while comments are present \
without forced writing (--force)"
                );
            }
        }
        Ok(())
    }

    fn fmt_possible_prefixes(
        &mut self,
        nodes: &mut Vec<(Node<'_>, Vec<Node<'_>>)>,
        context: &mut RootContext,
    ) -> Result<()> {
        if nodes.is_empty() {
            return Ok(());
        }
        if *context != RootContext::Start {
            writeln!(self.output)?;
        }
        if *context == RootContext::Triples {
            writeln!(self.output)?;
        }
        nodes.sort_by_key(|(node, _)| {
            node.child_by_field_name("label")
                .map_or("", |n| n.utf8_text(self.file).unwrap_or(""))
        });
        for (i, (node, comments)) in nodes.iter().enumerate() {
            if i > 0 {
                writeln!(self.output)?;
            }
            debug_assert_eq!(NodeKind::from(node), NodeKind::Prefix);
            self.fmt_prefix(*node)?;
            self.fmt_comments(comments.iter().copied(), true)?;
        }
        nodes.clear();
        *context = RootContext::Prefixes;
        Ok(())
    }

    fn fmt_base(&mut self, node: Node<'_>) -> Result<()> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::Base);
        let mut comments = Vec::new();
        for child in Self::iter_children(node)? {
            match NodeKind::from(&child) {
                NodeKind::Comment => comments.push(child),
                NodeKind::IriRef => {
                    let iri = self.extract_iri_ref(child)?;
                    write!(self.output, "@base <{iri}>")?;
                }
                _ => bail!("Unexpected base child: {}", child.to_sexp()),
            }
        }
        write!(self.output, " .")?;
        self.fmt_comments(comments, true)
    }

    fn fmt_prefix(&mut self, node: Node<'_>) -> Result<()> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::Prefix);
        let mut comments = Vec::new();
        let mut prefix = "";
        for child in Self::iter_children(node)? {
            match NodeKind::from(&child) {
                NodeKind::Comment => comments.push(child),
                NodeKind::PNPrefix => {
                    prefix = child.utf8_text(self.file)?;
                }
                NodeKind::IriRef => {
                    let iri = self.extract_iri_ref(child)?;
                    write!(self.output, "@prefix {prefix}: <{iri}>")?;
                    self.prefixes.insert(prefix.to_string(), iri);
                }
                _ => bail!("Unexpected prefix child: {}", child.to_sexp()),
            }
        }
        write!(self.output, " .")?;
        self.fmt_comments(comments, true)
    }

    fn new_indented_line(&mut self, indents: usize) -> Result<()> {
        writeln!(self.output)?;
        for _ in 0..(self.options.indentation * indents) {
            write!(self.output, " ")?;
        }
        Ok(())
    }

    fn fmt_triples(&mut self, node: Node<'_>) -> Result<()> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::Triples);
        let mut comments = Vec::new();
        let mut is_first_predicate_objects = true;
        let children = self.iter_children_sorted(
            node,
            self.options.sort_terms,
            |n| NodeKind::from(&n) == NodeKind::PredicateObjects,
            |n| n.child_by_field_name("predicate"),
        )?;
        for child in children {
            match NodeKind::from(&child) {
                NodeKind::Comment => comments.push(child),
                NodeKind::PredicateObjects => {
                    let new_line = if is_first_predicate_objects {
                        if !self.options.new_lines_for_easy_diff {
                            write!(self.output, " ")?;
                        }
                        is_first_predicate_objects = false;
                        self.options.new_lines_for_easy_diff
                    } else {
                        write!(self.output, " ;")?;
                        true
                    };
                    if new_line {
                        self.fmt_comments(comments.drain(0..), true)?;
                        self.new_indented_line(1)?;
                    }
                    self.fmt_predicate_objects(child, &mut comments, 1)?;
                }
                NodeKind::IriRef
                | NodeKind::PrefixedName
                | NodeKind::BlankNodePropertyList
                | NodeKind::BlankNodeLabel
                | NodeKind::Collection
                | NodeKind::BlankNodeAnonymous => {
                    // The subject
                    self.fmt_term(child, &mut comments, false, 0)?;
                }
                _ => {
                    bail!("Unexpected triples child kind: {}", child.kind());
                }
            }
        }
        if self.options.new_lines_for_easy_diff {
            write!(self.output, " ;")?;
            self.new_indented_line(1)?;
            write!(self.output, ".")?;
        } else {
            write!(self.output, " .")?;
        }
        self.fmt_comments(comments, true)
    }

    fn fmt_predicate_objects<'b>(
        &mut self,
        node: Node<'b>,
        comments: &mut Vec<Node<'b>>,
        indent_level: usize,
    ) -> Result<()> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::PredicateObjects);
        let mut is_predicate = true;
        let mut is_first_object = true;
        let num_objects = Self::iter_children(node)?
            .into_iter()
            .filter(|child| NodeKind::from(child) != NodeKind::Comment)
            .count()
            - 1;
        let mut seen_predicate = false;
        let children = self.iter_children_sorted(
            node,
            self.options.sort_terms && num_objects > 0,
            |n| {
                if NodeKind::from(&n) == NodeKind::Comment {
                    return false;
                }
                if !seen_predicate {
                    seen_predicate = true;
                    return false;
                }
                seen_predicate
            },
            |n| Some(n),
        )?;
        for child in children {
            match NodeKind::from(&child) {
                NodeKind::Comment => comments.push(child),
                _ => {
                    if is_predicate {
                        self.fmt_term(child, comments, true, indent_level + 1)?;
                        is_predicate = false;
                    } else {
                        if is_first_object {
                            if self.options.single_object_on_new_line
                                || (num_objects > 1 && self.options.new_lines_for_easy_diff)
                            {
                                self.new_indented_line(indent_level + 1)?;
                            } else {
                                write!(self.output, " ")?;
                            }
                            is_first_object = false;
                        } else if self.options.new_lines_for_easy_diff {
                            write!(self.output, " ,")?;
                            self.new_indented_line(indent_level + 1)?;
                        } else {
                            write!(self.output, " , ")?;
                        }
                        self.fmt_term(child, comments, false, indent_level + 1)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn fmt_term<'b>(
        &mut self,
        node: Node<'b>,
        comments: &mut Vec<Node<'b>>,
        is_predicate: bool,
        indent_level: usize,
    ) -> Result<()> {
        enum LiteralAnnotation {
            None,
            LangTag(String),
            IriRef(String),
            PrefixedName(String, String),
        }

        match NodeKind::from(&node) {
            NodeKind::IriRef => {
                let iri = self.extract_iri_ref(node)?;
                if is_predicate && iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                    write!(self.output, "a")
                } else {
                    write!(self.output, "<{iri}>")
                }?;
            }
            NodeKind::PrefixedName => {
                let ((prefix, local), iri) = self.extract_prefixed_name(node)?;
                if is_predicate && iri == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
                    write!(self.output, "a")
                } else {
                    write!(self.output, "{prefix}:{local}")
                }?;
            }
            NodeKind::A => write!(self.output, "a")?,
            NodeKind::BlankNodeAnonymous => write!(self.output, "[]")?,
            NodeKind::BlankNodeLabel => write!(self.output, "_:{}", node.utf8_text(self.file)?)?,
            NodeKind::BlankNodePropertyList => {
                let mut is_first_predicate_objects = true;
                write!(self.output, "[")?;
                let children = self.iter_children_sorted(
                    node,
                    self.options.sort_terms,
                    |n| NodeKind::from(&n) == NodeKind::PredicateObjects,
                    |n| n.child_by_field_name("predicate"),
                )?;
                for child in children {
                    if NodeKind::from(&child) == NodeKind::Comment {
                        comments.push(child);
                    } else {
                        let new_line = if is_first_predicate_objects {
                            is_first_predicate_objects = false;
                            self.options.new_lines_for_easy_diff
                        } else {
                            write!(self.output, " ;")?;
                            true
                        } && self.options.new_lines_for_easy_diff;
                        if new_line {
                            self.fmt_comments(comments.drain(0..), true)?;
                            self.new_indented_line(indent_level + 1)?;
                        } else {
                            write!(self.output, " ")?;
                        }
                        self.fmt_predicate_objects(child, comments, indent_level + 1)?;
                    }
                }
                if self.options.new_lines_for_easy_diff {
                    write!(self.output, " ;")?;
                    self.new_indented_line(indent_level)?;
                } else {
                    write!(self.output, " ")?;
                }
                write!(self.output, "]")?;
            }
            NodeKind::Collection => {
                write!(self.output, "(")?;
                let new_line = self.options.new_lines_for_easy_diff;
                // let new_line = true;
                for child in Self::iter_children(node)? {
                    if NodeKind::from(&child) == NodeKind::Comment {
                        comments.push(child);
                    } else {
                        if new_line {
                            self.new_indented_line(indent_level + 1)?;
                        } else {
                            write!(self.output, " ")?;
                        }
                        self.fmt_term(child, comments, false, indent_level + 1)?;
                    }
                }
                if new_line {
                    self.new_indented_line(indent_level)?;
                } else {
                    write!(self.output, " ")?;
                }
                write!(self.output, ")")?;
            }
            NodeKind::Literal => {
                let mut value = String::new();
                let mut is_long_string = false;
                let mut annotation = LiteralAnnotation::None;
                let mut datatype = Cow::Borrowed("http://www.w3.org/2001/XMLSchema#string");
                for child in Self::iter_children(node)? {
                    match NodeKind::from(&child) {
                        NodeKind::Comment => comments.push(child),
                        NodeKind::StringLiteral => {
                            (value, is_long_string) = self.extract_string(child)?;
                        }
                        NodeKind::LangTag => {
                            annotation =
                                LiteralAnnotation::LangTag(child.utf8_text(self.file)?.to_string());
                            datatype =
                                "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString".into();
                        }
                        NodeKind::IriRef => {
                            let iri_ref = self.extract_iri_ref(child)?;
                            annotation = LiteralAnnotation::IriRef(iri_ref.clone());
                            datatype = iri_ref.into();
                        }
                        NodeKind::PrefixedName => {
                            let ((prefix, local), resolved_iri) =
                                self.extract_prefixed_name(child)?;
                            annotation = LiteralAnnotation::PrefixedName(prefix, local);
                            datatype = resolved_iri.into();
                        }
                        NodeKind::At
                        | NodeKind::DataType
                        | NodeKind::IriStart
                        | NodeKind::IriEnd => (),
                        _ => bail!("Unexpected literal child: {}", child.to_sexp()),
                    }
                }
                match datatype.as_ref() {
                    "http://www.w3.org/2001/XMLSchema#boolean"
                        if matches!(value.as_str(), "true" | "false") =>
                    {
                        write!(self.output, "{value}")
                    }
                    "http://www.w3.org/2001/XMLSchema#integer" if is_turtle_integer(&value) => {
                        write!(self.output, "{value}")
                    }
                    "http://www.w3.org/2001/XMLSchema#decimal" if is_turtle_decimal(&value) => {
                        write!(self.output, "{value}")
                    }
                    "http://www.w3.org/2001/XMLSchema#double" if is_turtle_double(&value) => {
                        write!(self.output, "{value}")
                    }
                    _ => {
                        if is_long_string {
                            write!(self.output, "\"\"\"{value}\"\"\"")?;
                        } else {
                            write!(self.output, "\"{value}\"")?;
                        }
                        match annotation {
                            LiteralAnnotation::None => Ok(()),
                            LiteralAnnotation::LangTag(l) => write!(self.output, "@{l}"),
                            LiteralAnnotation::IriRef(i) => write!(self.output, "^^<{i}>"),
                            LiteralAnnotation::PrefixedName(prefix, local) => {
                                write!(self.output, "^^{prefix}:{local}")
                            }
                        }
                    }
                }?;
            }
            NodeKind::IntegerLiteral => {
                let value = node.utf8_text(self.file)?;
                debug_assert!(is_turtle_integer(value), "'{value}' should be an integer");
                write!(self.output, "{value}")?;
            }
            NodeKind::BooleanLiteral => {
                let value = node.utf8_text(self.file)?;
                debug_assert!(
                    matches!(value, "true" | "false"),
                    "'{value}' should rather be true or false"
                );
                write!(self.output, "{value}")?;
            }
            NodeKind::DecimalLiteral => {
                let value = node.utf8_text(self.file)?;
                debug_assert!(is_turtle_decimal(value), "{value} should be a decimal");
                write!(self.output, "{value}")?;
            }
            NodeKind::DoubleLiteral => {
                let value = node.utf8_text(self.file)?;
                debug_assert!(is_turtle_double(value), "{value} should be a double");
                write!(self.output, "{value}")?;
            }
            _ => bail!("Unexpected term: {}", node.to_sexp()),
        }
        Ok(())
    }

    /// Returns the **normalized** (i.e. URL quoted) IRI of the given IRI reference.
    fn extract_iri_ref(&self, node: Node<'_>) -> Result<String> {
        debug_assert_eq!(NodeKind::from(&node), NodeKind::IriRef);
        // We normalize the IRI
        let raw = node.utf8_text(self.file)?;
        let mut normalized = String::with_capacity(raw.len());
        for c in StringDecoder::new(raw) {
            match c? {
                c @ ('\x00'..='\x20' | '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\') => {
                    bail!("The character '{c:?} is not allowed in IRIs")
                }
                c => normalized.push(c),
            }
        }
        Ok(normalized)
    }

    /// Returns the prefix, the **normalized** local part,
    /// and the resolved IRI of the given prefixed name.
    ///
    /// # Panics
    ///
    /// - If a prefix is used that is not yet defined
    /// - If the local part contains invalid characters
    fn extract_prefixed_name(&self, node: Node<'_>) -> Result<((String, String), String)> {
        let (prefix, local) = node.utf8_text(self.file)?.split_once(':').unwrap();
        let Some(prefix_value) = self.prefixes.get(prefix) else {
            bail!(
                "The prefix {prefix}: is not defined on line {}",
                node.start_position().row + 1
            );
        };

        let mut normalized_local = String::with_capacity(local.len());
        let mut in_escape = false;
        for c in local.chars() {
            if in_escape {
                match c {
                    '_' => normalized_local.push(c),
                    '.' | '-' => {
                        if normalized_local.is_empty() {
                            normalized_local.push('\\');
                        }
                        normalized_local.push(c);
                    }
                    '~' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
                    | '/' | '?' | '#' | '@' | '%' => {
                        normalized_local.push('\\');
                        normalized_local.push(c);
                    }
                    c => bail!("Unexpected escape character \\{c}"),
                }
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else {
                normalized_local.push(c);
            }
        }
        if normalized_local.ends_with('.') {
            // We are not allowed to end with '.'
            normalized_local.pop();
            normalized_local.push_str("\\.");
        }

        let resolved = format!("{prefix_value}{normalized_local}");
        Ok(((prefix.to_string(), normalized_local), resolved))
    }

    /// Converts a parsed string literal node into a string
    /// representing that string literal in Turtle.
    ///
    /// That output string is ready to be printed into a `*.ttl` file,
    /// though without language tag or datatype.
    /// The second output indicates whether it is a multi-line string.
    fn extract_string(&self, string_lit_node: Node<'_>) -> Result<(String, bool)> {
        debug_assert_eq!(NodeKind::from(&string_lit_node), NodeKind::StringLiteral);

        let raw = string_lit_node.utf8_text(self.file)?;
        if raw.starts_with("\"\"\"") || raw.starts_with("'''") {
            // We normalize the multi-lines string
            let mut raw = &raw[3..raw.len() - 3];
            let mut normalized = String::with_capacity(raw.len());
            // Hack: double quotes at the end should be escaped
            let mut number_of_end_double_quotes = 0;
            loop {
                if raw.ends_with("\\\"") {
                    raw = &raw[..raw.len() - 2];
                    number_of_end_double_quotes += 1;
                } else if raw.ends_with('"') {
                    raw = &raw[..raw.len() - 1];
                    number_of_end_double_quotes += 1;
                } else {
                    break;
                }
            }
            let mut previous_double_quotes = 0;
            for c in StringDecoder::new(raw) {
                match c? {
                    '"' => {
                        if previous_double_quotes >= 2 {
                            normalized.push_str("\\\"");
                        } else {
                            normalized.push('"');
                            previous_double_quotes += 1;
                        }
                    }
                    '\\' => {
                        normalized.push_str("\\\\");
                        previous_double_quotes = 0;
                    }
                    c => {
                        normalized.push(c);
                        previous_double_quotes = 0;
                    }
                }
            }
            for _ in 0..number_of_end_double_quotes {
                normalized.push_str("\\\"");
            }

            Ok((normalized, true))
        } else {
            // We normalize the one-line string
            let raw = &raw[1..raw.len() - 1];
            let mut normalized = String::with_capacity(raw.len());
            for c in StringDecoder::new(raw) {
                match c? {
                    '"' => {
                        normalized.push_str("\\\"");
                    }
                    '\\' => {
                        normalized.push_str("\\\\");
                    }
                    '\r' => {
                        normalized.push_str("\\r");
                    }
                    '\n' => {
                        normalized.push_str("\\n");
                    }
                    '\t' => {
                        normalized.push_str("\\t");
                    }
                    c => normalized.push(c),
                }
            }

            Ok((normalized, false))
        }
    }

    /// Writes out the supplied comments.
    fn fmt_comments<'b>(
        &mut self,
        comment_nodes: impl IntoIterator<Item = Node<'b>>,
        inline: bool,
    ) -> Result<()> {
        let comments = comment_nodes
            .into_iter()
            .map(|node| Ok(node.utf8_text(self.file)?[1..].trim_end()))
            .collect::<Result<Vec<_>>>()?;
        if !comments.is_empty() {
            if self.options.includes_sorting() {
                self.seen_comments = true;
            }
            if inline {
                write!(self.output, " ")?;
            }
            write!(self.output, "#{}", comments.join(" "))?;
        }
        Ok(())
    }

    /// Returns a vector of the _named_ children of the given node.
    ///
    /// # Errors
    ///
    /// - if any of the children is a syntax error
    /// - if there is a missing node
    ///   (created by the parser to recover from an invalid state)
    fn iter_children(node: Node<'_>) -> Result<Vec<Node<'_>>> {
        let mut walk = node.walk();
        node.children(&mut walk)
            .filter_map(|child| {
                if child.is_error() || child.is_missing() {
                    Some(Err(Self::fmt_err(child)))
                } else if child.is_named() {
                    Some(Ok(child))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Sort nodes in-place, based on their sort key (a sub-node),
    /// which is extracted from each node in the list to be sorted
    /// through the given function.
    ///
    /// If the return of the sort key is `None` -
    /// it failed to be extracted from a node -
    /// the node is not sorted,
    /// or say,
    /// it is put at the end or the beginning of the (sorted) output,
    /// together in a bunch with the other unsortable nodes.
    fn sort_nodes<KS: Fn(Node<'_>) -> Option<Node<'_>>>(
        &self,
        to_be_sorted: &mut [Node<'_>],
        extract_sort_key_sub_node: KS,
    ) {
        to_be_sorted.sort_by_key(|n| {
            extract_sort_key_sub_node(*n).map_or((NodeKind::None, ""), |n| {
                (NodeKind::from(&n), n.utf8_text(self.file).unwrap_or(""))
            })
        });
    }

    /// Iterates through the children of the given node,
    /// optionally first sorting them..
    ///
    /// If sorting is enabled,
    /// that sorting does not happen globally,
    /// but within sections limited by:
    ///
    /// - the beginning of the list
    /// - `NodeKind::Base`
    /// - `NodeKind::Prefix`
    /// - the end of the list
    ///
    /// Furthermore, only nodes selected by  `is_to_be_sorted` will be sorted,
    /// while the others will be yielded at the beginning of the section
    /// in the order they appear in the input.
    fn iter_children_sorted<
        'i,
        CS: FnMut(Node<'_>) -> bool,
        KS: Fn(Node<'_>) -> Option<Node<'_>>,
    >(
        &self,
        node: Node<'i>,
        sort: bool,
        mut is_to_be_sorted: CS,
        extract_sort_key_sub_node: KS,
    ) -> Result<Vec<Node<'i>>> {
        let children = if sort {
            let mut sorted = vec![];
            let mut to_be_sorted = vec![];
            for child in Self::iter_children(node)? {
                let child_kind = NodeKind::from(&child);
                if child_kind == NodeKind::Base || child_kind == NodeKind::Prefix {
                    self.sort_nodes(&mut to_be_sorted, &extract_sort_key_sub_node);
                    sorted.append(&mut to_be_sorted);
                    to_be_sorted.clear();
                }
                if is_to_be_sorted(child) {
                    to_be_sorted.push(child);
                } else {
                    sorted.push(child);
                }
            }
            self.sort_nodes(&mut to_be_sorted, extract_sort_key_sub_node);
            sorted.append(&mut to_be_sorted);
            sorted
        } else {
            Self::iter_children(node)?
        };
        Ok(children)
    }

    /// Creates an anonymous anyhow error for a node,
    /// including the nodes context (line and column numbers) within the input.
    fn fmt_err(node: Node<'_>) -> Error {
        let start = node.start_position();
        let end = node.end_position();
        if start.row == end.row {
            anyhow!(
                "Error on line {} between bytes {} and {}: {}",
                start.row + 1,
                start.column + 1,
                end.column + 1,
                node.to_sexp()
            )
        } else {
            anyhow!(
                "Error between lines {} and {}: {}",
                start.row + 1,
                end.row + 1,
                node.to_sexp()
            )
        }
    }
}
