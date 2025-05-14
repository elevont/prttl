// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use crate::ast::{
    construct_tree, SortingContext, TBlankNode, TBlankNodeRef, TCollection, TLiteralRef, TObject,
    TPredicateCont, TRoot, TSubject, TSubjectCont,
};
use crate::context::Context;
use crate::options::FormatOptions;
use crate::parser;
use oxrdf::{vocab::rdf, vocab::xsd, BlankNodeRef, NamedNode, NamedNodeRef};
use regex::Regex;
use std::fmt::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::LazyLock;
use thiserror::Error;

use crate::input::Input;

#[derive(Debug)]
pub enum FilesListErrorType {
    ReadDir,
    ExtractEntry,
    EvaluateFileType,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("We do not support redefinition of prefixes, which is the case with {0}")]
    PrefixRedefinition(String),

    #[error("We do not support more then one base IRI defined per file")]
    MultipleBases,

    #[error(transparent)]
    TurtleSyntaxError(#[from] oxttl::TurtleSyntaxError),

    /// Represents all cases of `std::io::Error`.
    #[error(transparent)]
    Format(#[from] std::fmt::Error),

    #[error("The target to format {0} does not seem to exist")]
    TargetFileDoesNotExist(PathBuf),

    #[error("Error while reading {0}")]
    FailedToReadTargetFile(PathBuf),

    #[error("Failed to parse input as turtle: {0}")]
    ParseError(#[from] parser::Error),

    #[error("Error while writing {0}")]
    FailedToWriteFormattedFile(PathBuf),

    #[error("Failed to list files in input directory {0}: {1:?}")]
    FailedToListFilesInInputDir(PathBuf, FilesListErrorType),

    #[error("Failed to create Turtle file tree structure: {0}")]
    FailedToCreateTurtleStructure(String),
}

pub type Result<T> = std::result::Result<T, Error>;

static RE_NAMESPACE_DIVIDER: LazyLock<Regex> = LazyLock::new(|| Regex::new("[/#]").unwrap());

pub fn format(input: &Input, options: Rc<FormatOptions>) -> Result<String> {
    let mut output = String::new();
    let mut context = Context {
        indent_level: 0,
        output: &mut output,
    };
    let mut formatter = TurtleFormatter::new(input, options);
    formatter.construct_tree();
    // println!("{:#?}", formatter.tree);
    formatter.fmt_doc(&mut context)?;
    Ok(output)
}

struct TurtleFormatter<'graph> {
    input: &'graph Input,
    options: Rc<FormatOptions>,
    tree: TRoot<'graph>,
}

impl<'graph> TurtleFormatter<'graph> {
    const fn new(input: &'graph Input, options: Rc<FormatOptions>) -> Self {
        Self {
            input,
            options,
            tree: TRoot::new(),
        }
    }

    fn construct_tree(&mut self) {
        construct_tree(&mut self.tree, self.input)
            .map_err(|err| Error::FailedToCreateTurtleStructure(err.to_string()))
            .unwrap();

        let context = SortingContext {
            options: Rc::<_>::clone(&self.options),
            graph: &self.input.graph,
        };
        self.tree.sort(&context);
    }
}

impl<'graph> TurtleFormatter<'graph> {
    fn fmt_base<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        let base_iri = if let Some(base_iri) = self.input.base.as_deref() {
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

    fn fmt_prefixes<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        for (prefix, iri) in &self.input.prefixes {
            if self.options.sparql_syntax {
                writeln!(context.output, "PREFIX {prefix}: <{iri}>")?;
            } else {
                writeln!(context.output, "@prefix {prefix}: <{iri}> .")?;
            }
        }
        Ok(())
    }

    fn write_indent<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        for _ in 0..context.indent_level {
            write!(context.output, "{}", self.options.indentation)?;
        }
        Ok(())
    }

    fn fmt_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        named_node: &NamedNodeRef<'_>,
    ) -> Result<()> {
        self.write_indent(context)?;

        if *named_node == rdf::TYPE {
            write!(context.output, "a")?;
            return Ok(());
        }

        let iri: &str = named_node.as_str();

        if let Some(base_iri) = self.input.base.as_deref() {
            if let Some(baseless_iri) = iri.strip_prefix(base_iri) {
                write!(context.output, "<{baseless_iri}>")?;
                return Ok(());
            }
        }

        let local_name = RE_NAMESPACE_DIVIDER.split(iri).last().unwrap();
        let iri: &str = named_node.as_str();
        let namespace = &iri[0..(iri.len() - local_name.len())];
        if let Some(prefix) = self.input.prefixes_inverted.get(namespace) {
            write!(context.output, "{prefix}:{local_name}")?;
        } else {
            write!(context.output, "<{iri}>")?;
        }
        Ok(())
    }

    fn fmt_blank_node_label<W: Write>(
        &self,
        context: &mut Context<W>,
        blank_node: &BlankNodeRef<'_>,
    ) -> Result<()> {
        self.write_indent(context)?;
        write!(context.output, "{blank_node}")?;
        Ok(())
    }

    fn fmt_blank_node_anonymous<W: Write>(
        &self,
        context: &mut Context<W>,
        blank_node: &TBlankNode<'graph>,
    ) -> Result<()> {
        self.write_indent(context)?;
        write!(context.output, "[")?;
        self.fmt_predicates(context, &blank_node.predicates, false)?;
        write!(context.output, "]")?;
        Ok(())
    }

    fn fmt_collection<W: Write>(
        &self,
        context: &mut Context<W>,
        collection: &TCollection<'graph>,
    ) -> Result<()> {
        self.write_indent(context)?;
        write!(context.output, "(")?;
        match collection {
            TCollection::Empty => (),
            TCollection::WithContent(collection_ref) => {
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
        write!(context.output, ")")?;
        Ok(())
    }

    fn fmt_literal<W: Write>(
        &self,
        context: &mut Context<W>,
        literal: &TLiteralRef<'graph>,
    ) -> Result<()> {
        self.write_indent(context)?;
        if literal.0.is_plain() {
            write!(context.output, "{}", literal.0)?;
        } else {
            match literal.0.datatype() {
                xsd::BOOLEAN
                | xsd::FLOAT
                | xsd::INTEGER
                | xsd::STRING
                | xsd::DECIMAL
                | xsd::DOUBLE
                | xsd::LONG
                | xsd::INT
                | xsd::SHORT
                | xsd::BYTE => write!(context.output, "{}", literal.0.value())?,
                dt => {
                    write!(context.output, "\"{}\"^^", literal.0.value())?;
                    let bak_indent = context.indent_level;
                    context.indent_level = 0;
                    self.fmt_named_node(context, &dt)?;
                    context.indent_level = bak_indent;
                }
            }
        }
        Ok(())
    }

    fn fmt_obj<W: Write>(&self, context: &mut Context<W>, obj: &TObject<'graph>) -> Result<()> {
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
            TObject::Triple(_) => todo!(),
        }
        Ok(())
    }

    fn fmt_subj<W: Write>(&self, context: &mut Context<W>, subj: &TSubject<'graph>) -> Result<()> {
        match subj {
            TSubject::NamedNode(named_node_ref) => self.fmt_named_node(context, named_node_ref)?,
            TSubject::BlankNodeLabel(TBlankNodeRef(blank_node_ref)) => {
                self.fmt_blank_node_label(context, blank_node_ref)?;
            }
            TSubject::BlankNodeAnonymous(blank_node) => {
                self.fmt_blank_node_anonymous(context, blank_node)?;
            }
            TSubject::Collection(collection) => self.fmt_collection(context, collection)?,
            TSubject::Triple(_) => todo!(),
        }
        Ok(())
    }

    fn fmt_subj_cont<W: Write>(
        &self,
        context: &mut Context<W>,
        subj_cont: &TSubjectCont<'graph>,
    ) -> Result<()> {
        self.fmt_subj(context, &subj_cont.subject)?;
        self.fmt_predicates(context, &subj_cont.predicates, true)?;
        writeln!(context.output)?;
        Ok(())
    }

    fn fmt_predicates<W: Write>(
        &self,
        context: &mut Context<W>,
        predicates_conts: &Vec<TPredicateCont<'graph>>,
        final_dot: bool,
    ) -> Result<()> {
        if !predicates_conts.is_empty() {
            writeln!(context.output)?;
            context.indent_level += 1;
            for predicates_cont in predicates_conts {
                self.fmt_named_node(context, &predicates_cont.predicate.0)?;
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
        Ok(())
    }

    fn fmt_triples<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        for subj_cont in &self.tree.subjects {
            self.fmt_subj_cont(context, subj_cont)?;
        }
        Ok(())
    }

    fn fmt_doc<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        self.fmt_base(context)?;

        self.fmt_prefixes(context)?;

        writeln!(context.output)?;

        self.fmt_triples(context)?;
        Ok(())
    }
}
