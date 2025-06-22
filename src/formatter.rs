// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use crate::ast::Part;
use crate::ast::{
    construct_tree, SortingContext, TBlankNode, TBlankNodeRef, TCollection, TLiteralRef,
    TNamedNode, TObject, TPredicateCont, TRoot, TSubject, TSubjectCont, TTriple,
};
use crate::constants::SUBSTITUTE_BASE;
use crate::context::Context;
use crate::error::Error;
use crate::error::FmtResult;
use crate::options::FormatOptions;
use oxiri::IriParseError;
use oxrdf::NamedNode;
use oxrdf::{vocab::rdf, vocab::xsd, BlankNodeRef, NamedNodeRef};
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write};
use std::rc::Rc;
use std::sync::LazyLock;

use crate::input::Input;

/// The regex to match a DOUBLE from the Turtle grammar,
/// which is *not* equivalent with xsd:double!
static RE_TURTLE_DOUBLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[+-]?(([0-9]+([.][0-9]*)?)|([.][0-9]+))[eE][+-]?[0-9]+").unwrap());

/// Does the actual formatting/pretty-printing.
///
/// # Errors
///
/// Fails only if an I/O error occurs while writing to an in-memory buffer.
/// This is basically only possible when the buffer is full,
/// and no more memory can be allocated.
pub fn format(input: &Input, options: Rc<FormatOptions>) -> FmtResult<String> {
    let mut output = String::new();
    let mut context = Context {
        indent_level: 0,
        output: &mut output,
    };
    let mut formatter = TurtleFormatter::new(input, options);
    formatter.construct_tree();
    tracing::debug!("{:#?}", formatter.tree);
    formatter.fmt_doc(&mut context)?;
    Ok(output)
}

struct TurtleFormatter<'graph> {
    input: &'graph Input,
    options: Rc<FormatOptions>,
    unreferenced_blank_nodes: HashSet<BlankNodeRef<'graph>>,
    tree: TRoot<'graph>,
}

impl<'graph> TurtleFormatter<'graph> {
    fn new(input: &'graph Input, options: Rc<FormatOptions>) -> Self {
        Self {
            input,
            options,
            unreferenced_blank_nodes: HashSet::new(),
            tree: TRoot::new(),
        }
    }

    fn try_named_node_from_iri_or_prefixed_name<'fleet>(
        input: &'graph Input,
        iri_or_name: &'fleet str,
    ) -> Result<NamedNode, IriParseError> {
        if let Some((prefix, local_name)) = iri_or_name.split_once(':') {
            if let Some(namespace) = input.prefixes.get(prefix) {
                return NamedNode::new(format!("{namespace}{local_name}"));
            }
        }
        NamedNode::new(iri_or_name)
    }

    fn construct_tree(&mut self) {
        construct_tree(
            &mut self.tree,
            &mut self.unreferenced_blank_nodes,
            self.input,
        )
        .map_err(|err| Error::FailedToCreateTurtleStructure(err.to_string()))
        .unwrap();

        let context = SortingContext {
            options: Rc::<_>::clone(&self.options),
            graph: &self.input.graph,
            bn_sorting_ids: Rc::new(RefCell::new(HashMap::new())),
            predicate_order: self
                .options
                .predicate_order()
                .into_iter()
                .enumerate()
                .map(|(idx, val)| {
                    let absolute_iri =
                        Self::try_named_node_from_iri_or_prefixed_name(self.input, &val)
                            .unwrap_or_else(|_| panic!("Failed to parse '{val}' as named node"))
                            .as_str()
                            .to_string();
                    (absolute_iri, idx)
                })
                .collect(),
            subject_type_order: self.options.subject_type_order().map(|names| {
                names
                    .into_iter()
                    .enumerate()
                    .map(|(idx, val)| {
                        (
                            Self::try_named_node_from_iri_or_prefixed_name(self.input, &val)
                                .unwrap_or_else(|_| panic!("Failed to parse '{val}' as named node"))
                                .as_str()
                                .to_string(),
                            idx,
                        )
                    })
                    .collect()
            }),
        };
        self.tree.sort(&context);
    }
}

fn escape_local_name(value: &str) -> Option<String> {
    // TODO: PLX
    // [168s] 	PN_LOCAL 	::= 	(PN_CHARS_U | ':' | [0-9] | PLX) ((PN_CHARS | '.' | ':' | PLX)* (PN_CHARS | ':' | PLX))?
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    let first = chars.next()?;
    if is_possible_pn_chars_u(first) || first == ':' || first.is_ascii_digit() {
        output.push(first);
    } else if can_be_escaped_in_local_name(first) {
        output.push('\\');
        output.push(first);
    } else {
        tracing::debug!("Can not escape (first) char in local name: '{first}'");
        return None;
    }

    while let Some(c) = chars.next() {
        if is_possible_pn_chars(c) || c == ':' || (c == '.' && !chars.as_str().is_empty()) {
            output.push(c);
        } else if can_be_escaped_in_local_name(c) {
            output.push('\\');
            output.push(c);
        } else {
            tracing::debug!("Can not escape char in local name: '{c}'");
            return None;
        }
    }

    Some(output)
}

// [157s]  PN_CHARS_BASE  ::=  [A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] | [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]
const fn is_possible_pn_chars_base(c: char) -> bool {
    matches!(c,
        'A'..='Z'
        | 'a'..='z'
        | '\u{00C0}'..='\u{00D6}'
        | '\u{00D8}'..='\u{00F6}'
        | '\u{00F8}'..='\u{02FF}'
        | '\u{0370}'..='\u{037D}'
        | '\u{037F}'..='\u{1FFF}'
        | '\u{200C}'..='\u{200D}'
        | '\u{2070}'..='\u{218F}'
        | '\u{2C00}'..='\u{2FEF}'
        | '\u{3001}'..='\u{D7FF}'
        | '\u{F900}'..='\u{FDCF}'
        | '\u{FDF0}'..='\u{FFFD}'
        | '\u{10000}'..='\u{EFFFF}')
}

// [158s]  PN_CHARS_U  ::=  PN_CHARS_BASE | '_'
pub(super) const fn is_possible_pn_chars_u(c: char) -> bool {
    is_possible_pn_chars_base(c) || c == '_'
}

// [160s]  PN_CHARS  ::=  PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
pub(crate) const fn is_possible_pn_chars(c: char) -> bool {
    is_possible_pn_chars_u(c)
        || matches!(c,
        '-' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}')
}

const fn can_be_escaped_in_local_name(c: char) -> bool {
    matches!(
        c,
        '_' | '~'
            | '.'
            | '-'
            | '!'
            | '$'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | ';'
            | '='
            | '/'
            | '?'
            | '#'
            | '@'
            | '%'
    )
}

impl<'graph> TurtleFormatter<'graph> {
    fn fmt_base<W: Write>(&self, context: &mut Context<W>) -> FmtResult<()> {
        let base_iri = if let Some(base_iri) = self.input.base.as_deref() {
            if base_iri == SUBSTITUTE_BASE {
                return Ok(());
            }
            base_iri.to_owned()
        } else {
            return Ok(());
        };
        if self.options.sparql_syntax {
            writeln!(context.output, "BASE <{base_iri}>")?;
        } else {
            writeln!(context.output, "@base <{base_iri}> .")?;
        }
        Ok(())
    }

    fn fmt_prefixes<W: Write>(&self, context: &mut Context<W>) -> FmtResult<()> {
        for (prefix, iri) in &self.input.prefixes {
            if self.options.sparql_syntax {
                writeln!(context.output, "PREFIX {prefix}: <{iri}>")?;
            } else {
                writeln!(context.output, "@prefix {prefix}: <{iri}> .")?;
            }
        }
        Ok(())
    }

    fn write_indent<W: Write>(&self, context: &mut Context<W>) -> FmtResult<()> {
        for _ in 0..context.indent_level {
            write!(context.output, "{}", self.options.indentation)?;
        }
        Ok(())
    }

    fn fmt_prefixed_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        named_node: &NamedNodeRef<'_>,
        prefix: &str,
        local_name: &str,
    ) -> FmtResult<()> {
        self.write_indent(context)?;

        if *named_node == rdf::TYPE {
            write!(context.output, "a")?;
            return Ok(());
        }

        if local_name.is_empty() {
            write!(context.output, "{prefix}:")?;
        } else {
            write!(
                context.output,
                "{prefix}:{}",
                escape_local_name(local_name).expect("Failed to escape local name")
            )?;
        }
        Ok(())
    }

    fn fmt_based_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        _named_node: &NamedNodeRef<'_>,
        additional_name: &str,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        write!(context.output, "<{additional_name}>")?;
        Ok(())
    }

    fn fmt_plain_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        named_node: &NamedNodeRef<'_>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;

        if *named_node == rdf::TYPE {
            write!(context.output, "a")?;
            return Ok(());
        }

        let iri: &str = named_node.as_str();
        write!(context.output, "<{iri}>")?;
        Ok(())
    }

    fn fmt_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        named_node: &TNamedNode<'_>,
    ) -> FmtResult<()> {
        match named_node {
            TNamedNode::Plain(named_node_ref) => self.fmt_plain_named_node(context, named_node_ref),
            TNamedNode::Prefixed(named_node_ref, prefix, local_name) => {
                self.fmt_prefixed_named_node(context, named_node_ref, prefix, local_name)
            }
            TNamedNode::Based(named_node_ref, additional_name) => {
                self.fmt_based_named_node(context, named_node_ref, additional_name)
            }
        }
    }

    fn fmt_blank_node_label<W: Write>(
        &self,
        context: &mut Context<W>,
        blank_node: &BlankNodeRef<'_>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        if self.unreferenced_blank_nodes.contains(blank_node) {
            panic!("Unreferenced blank-node stored in tree as labeled; should be anonymous");
        } else {
            write!(context.output, "{blank_node}")?;
        }
        Ok(())
    }

    fn fmt_blank_node_anonymous<W: Write>(
        &self,
        context: &mut Context<W>,
        blank_node: &TBlankNode<'graph>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        write!(context.output, "[")?;
        self.fmt_predicates(context, &blank_node.predicates, false)?;
        write!(context.output, "]")?;
        Ok(())
    }

    fn fmt_triple<W: Write>(
        &self,
        context: &mut Context<W>,
        triple: &TTriple<'graph>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        // write!(context.output, "<<( ")?;
        write!(context.output, "<< ")?;
        self.fmt_subj(context, &triple.0)?;
        write!(context.output, " ")?;
        self.fmt_named_node(context, &triple.1)?;
        write!(context.output, " ")?;
        self.fmt_obj(context, &triple.2)?;
        // write!(context.output, " )>>")?;
        write!(context.output, " >>")?;
        Ok(())
    }

    fn fmt_collection<W: Write>(
        &self,
        context: &mut Context<W>,
        collection: &TCollection<'graph>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        write!(context.output, "(")?;
        match collection {
            TCollection::Empty => (),
            TCollection::WithContent(collection_ref) => {
                if !self.options.single_leafed_new_lines && collection.is_single_leafed() {
                    write!(context.output, " ")?;
                    let bak_indent = context.indent_level;
                    context.indent_level = 0;
                    self.fmt_obj(context, collection_ref.rest.first().unwrap())?;
                    context.indent_level = bak_indent;
                    write!(context.output, " ")?;
                } else {
                    writeln!(context.output)?;
                    context.indent_level += 1;
                    let mut first_entry = true;
                    for entry in &collection_ref.rest {
                        if first_entry {
                            first_entry = false;
                        } else {
                            writeln!(context.output)?;
                        }
                        self.fmt_obj(context, entry)?;
                    }
                    writeln!(context.output)?;
                    context.indent_level -= 1;
                    self.write_indent(context)?;
                }
            }
        }
        write!(context.output, ")")?;
        Ok(())
    }

    fn fmt_literal_with_type<W: Write>(
        &self,
        context: &mut Context<W>,
        literal: &TLiteralRef<'graph>,
    ) -> FmtResult<()> {
        write!(context.output, "\"{}\"^^", literal.0.value())?;
        let bak_indent = context.indent_level;
        context.indent_level = 0;
        let nice_dt = literal
            .1
            .as_ref()
            .expect("The TRoot generating algorithm failed to supply a datatype for a literal");
        self.fmt_named_node(context, nice_dt)?;
        context.indent_level = bak_indent;
        Ok(())
    }

    /// Copied from [`oxrdf::literal::print_quoted_str`]
    /// because it is not publicly exported there :/
    #[inline]
    pub fn print_quoted_str(string: &str, f: &mut impl Write) -> fmt::Result {
        f.write_char('"')?;
        for c in string.chars() {
            match c {
                '\u{08}' => f.write_str("\\b"),
                '\t' => f.write_str("\\t"),
                '\n' => f.write_str("\\n"),
                '\u{0C}' => f.write_str("\\f"),
                '\r' => f.write_str("\\r"),
                '"' => f.write_str("\\\""),
                '\\' => f.write_str("\\\\"),
                '\0'..='\u{1F}' | '\u{7F}' => write!(f, "\\u{:04X}", u32::from(c)),
                _ => f.write_char(c),
            }?;
        }
        f.write_char('"')
    }

    #[inline]
    pub fn print_unquoted_str(string: &str, f: &mut impl Write) -> fmt::Result {
        f.write_str("\"\"\"")?;
        let mut consecutive_quote_counter = 0;
        let mut prev_char_opt = None;
        for c in string.chars() {
            if let Some(prev_char) = prev_char_opt {
                f.write_char(prev_char)?;
            }
            if c == '"' {
                consecutive_quote_counter += 1;
                if consecutive_quote_counter == 3 {
                    f.write_char('\\')?;
                    consecutive_quote_counter = 0;
                }
            }
            prev_char_opt = Some(c);
        }
        if let Some(prev_char) = prev_char_opt {
            if prev_char == '"' {
                f.write_char('\\')?;
            }
            f.write_char(prev_char)?;
        }
        f.write_str("\"\"\"")
    }

    fn fmt_string<W: Write>(context: &mut Context<W>, value: &'graph str) -> FmtResult<()> {
        // NOTE We need to use quoted for strings containing "\n\r",
        //      because they can not be represented in triple-quoted strings.
        if value.contains('\n') && !value.contains("\n\r") {
            Self::print_unquoted_str(value, &mut context.output)?;
        } else {
            Self::print_quoted_str(value, &mut context.output)?;
        }
        Ok(())
    }

    fn fmt_literal<W: Write>(
        &self,
        context: &mut Context<W>,
        literal: &TLiteralRef<'graph>,
    ) -> FmtResult<()> {
        self.write_indent(context)?;
        match literal.0.datatype() {
            xsd::STRING => Self::fmt_string(context, literal.0.value())?,
            rdf::LANG_STRING => {
                Self::fmt_string(context, literal.0.value())?;
                write!(context.output, "@")?;
                write!(
                    context.output,
                    "{}",
                    literal
                        .0
                        .language()
                        .expect("langString should always have a language specified")
                )?;
            }
            xsd::BOOLEAN | xsd::INTEGER => write!(context.output, "{}", literal.0.value())?,
            xsd::DOUBLE => {
                if RE_TURTLE_DOUBLE.is_match(literal.0.value()) {
                    write!(context.output, "{}", literal.0.value())?;
                } else {
                    if self.options.warn_unsupported_numbers {
                        tracing::warn!(
                            "As pointed out in <https://github.com/w3c/rdf-turtle/issues/98>,
Not all valid xsd:double values can be written as Turtle `DOUBLE`s,
so we write them as data-typed literals."
                        );
                    }
                    self.fmt_literal_with_type(context, literal)?;
                }
            }
            xsd::DECIMAL => {
                if literal.0.value().ends_with('.') || !literal.0.value().contains('.') {
                    if self.options.warn_unsupported_numbers {
                        tracing::warn!(
                            "As pointed out in <https://github.com/w3c/rdf-turtle/issues/98>,
Not all valid xsd:decimal values can be written as Turtle `DECIMAL`s,
so we write them as data-typed literals."
                        );
                    }
                    self.fmt_literal_with_type(context, literal)?;
                } else {
                    write!(context.output, "{}", literal.0.value())?;
                }
            }
            _dt => self.fmt_literal_with_type(context, literal)?,
        }
        Ok(())
    }

    fn fmt_obj<W: Write>(&self, context: &mut Context<W>, obj: &TObject<'graph>) -> FmtResult<()> {
        match obj {
            TObject::NamedNode(named_node_ref) => self.fmt_named_node(context, named_node_ref)?,
            TObject::BlankNodeLabel(TBlankNodeRef(blank_node_ref)) => {
                self.fmt_blank_node_label(context, blank_node_ref)?;
            }
            TObject::BlankNodeAnonymous(blank_node) => {
                self.fmt_blank_node_anonymous(context, blank_node)?;
            }
            TObject::Collection(collection) => self.fmt_collection(context, collection)?,
            TObject::Literal(t_literal_ref) => self.fmt_literal(context, t_literal_ref)?,
            TObject::Triple(triple) => self.fmt_triple(context, triple)?,
        }
        Ok(())
    }

    fn fmt_subj<W: Write>(
        &self,
        context: &mut Context<W>,
        subj: &TSubject<'graph>,
        // top_level: bool,
    ) -> FmtResult<()> {
        match subj {
            TSubject::NamedNode(named_node_ref) => self.fmt_named_node(context, named_node_ref)?,
            TSubject::BlankNodeLabel(TBlankNodeRef(blank_node_ref)) => {
                self.fmt_blank_node_label(context, blank_node_ref)?;
            }
            TSubject::BlankNodeAnonymous(blank_node) => {
                self.fmt_blank_node_anonymous(context, blank_node)?;
            }
            TSubject::Collection(collection) => self.fmt_collection(context, collection)?,
            TSubject::Triple(triple) => self.fmt_triple(context, triple)?,
        }
        Ok(())
    }

    fn fmt_subj_cont<W: Write>(
        &self,
        context: &mut Context<W>,
        subj_cont: &TSubjectCont<'graph>,
    ) -> FmtResult<()> {
        self.fmt_subj(context, &subj_cont.subject)?;
        if !matches!(subj_cont.subject, TSubject::BlankNodeAnonymous(_)) {
            self.fmt_predicates(context, &subj_cont.predicates, true)?;
        }
        if matches!(
            subj_cont.subject,
            TSubject::BlankNodeAnonymous(_) | TSubject::Collection(_)
        ) {
            if matches!(subj_cont.subject, TSubject::BlankNodeAnonymous(_)) {
                write!(context.output, " .")?;
            }
            writeln!(context.output)?;
        }
        writeln!(context.output)?;
        Ok(())
    }

    fn fmt_predicates<W: Write>(
        &self,
        context: &mut Context<W>,
        predicates_containers: &Vec<TPredicateCont<'graph>>,
        final_dot: bool,
    ) -> FmtResult<()> {
        if !predicates_containers.is_empty() {
            if !self.options.single_leafed_new_lines
                && predicates_containers.len() == 1
                && predicates_containers.first().unwrap().is_single_leafed()
            {
                let predicates_cont = predicates_containers.first().unwrap();
                write!(context.output, " ")?;
                let bak_indent = context.indent_level;
                context.indent_level = 0;
                self.fmt_named_node(context, &predicates_cont.predicate)?;
                write!(context.output, " ")?;
                self.fmt_obj(context, predicates_cont.objects.first().unwrap())?;
                if final_dot {
                    write!(context.output, " .")?;
                } else {
                    write!(context.output, " ")?;
                }
                context.indent_level = bak_indent;
                // writeln!(context.output, " ;")?;
                // context.indent_level += 1;
            } else {
                writeln!(context.output)?;
                context.indent_level += 1;
                for predicates_cont in predicates_containers {
                    self.fmt_named_node(context, &predicates_cont.predicate)?;
                    if !self.options.single_leafed_new_lines && predicates_cont.is_single_leafed() {
                        write!(context.output, " ")?;
                        let bak_indent = context.indent_level;
                        context.indent_level = 0;
                        self.fmt_obj(context, predicates_cont.objects.first().unwrap())?;
                        context.indent_level = bak_indent;
                    } else {
                        context.indent_level += 1;
                        let mut first_obj = true;
                        for obj in &predicates_cont.objects {
                            if first_obj {
                                first_obj = false;
                                writeln!(context.output)?;
                            } else {
                                writeln!(context.output, " ,")?;
                            }
                            self.fmt_obj(context, obj)?;
                        }
                        context.indent_level -= 1;
                    }
                    writeln!(context.output, " ;")?;
                }
                if final_dot {
                    self.write_indent(context)?;
                    writeln!(context.output, ".")?;
                }
                context.indent_level -= 1;
                if !final_dot {
                    self.write_indent(context)?;
                }
            }
        }
        Ok(())
    }

    fn fmt_triples<W: Write>(&self, context: &mut Context<W>) -> FmtResult<()> {
        for subj_cont in &self.tree.subjects {
            self.fmt_subj_cont(context, subj_cont)?;
        }
        Ok(())
    }

    fn fmt_doc<W: Write>(&self, context: &mut Context<W>) -> FmtResult<()> {
        self.fmt_base(context)?;

        self.fmt_prefixes(context)?;

        writeln!(context.output)?;

        self.fmt_triples(context)?;
        Ok(())
    }
}
