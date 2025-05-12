// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use crate::ast::{
    construct_tree, TBlankNode, TBlankNodeRef, TCollection, TLiteralRef, TObject, TPredicateCont,
    TRoot, TSubject, TSubjectCont, NN_RDF_TYPE,
};
use crate::context::Context;
use crate::options::FormatOptions;
use crate::parser;
use oxrdf::{vocab::xsd, BlankNode, BlankNodeRef, Literal, NamedNode, NamedNodeRef, Subject, Term};
use regex::Regex;
use std::cmp::Ordering;
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

static PRTYR_SORTING_ID: LazyLock<NamedNode> =
    LazyLock::new(|| NamedNode::new("http://w3id.org/oseg/ont/prtyr#sortingId").unwrap());

static RE_NAMESPACE_DIVIDER: LazyLock<Regex> = LazyLock::new(|| Regex::new("[/#]").unwrap());

// struct PrefixedResource {
//     /// The prefix; e.g. `"xsd"` or `"schema"`.
//     pub prefix: String,
//     /// The resources local name, normalized; e.g. `"string"` or `"Person"`.
//     pub local: String,
//     /// The resolved resource;
//     /// e.g. `"http://www.w3.org/2001/XMLSchema#string"`
//     /// or `"http://schema.org/Person"`.
//     pub iri: String,
// }

pub fn format(input: &Input, options: Rc<FormatOptions>) -> Result<String> {
    let mut formatted = String::new();
    // let need_graph = options.prtyr_sorting;
    // let input = crate::parser::parse(original.as_bytes())?;
    // let graph = if need_graph {
    //     let mut graph = Graph::new();
    //     for triple in TurtleParser::new().for_reader(original.as_bytes()) {
    //         let triple = triple?;
    //         graph.insert(&triple);
    //     }
    //     Some(graph)
    // } else {
    //     None
    // };
    let mut context = Context {
        indent_level: 0,
        output: &mut formatted,
    };
    let mut formatter = TurtleFormatter::new(input, options);
    formatter.construct_tree();
    println!("{:#?}", formatter.tree);
    formatter.fmt_doc(&mut context)?;
    Ok(formatted)
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
            // prefixes: HashMap::new(),
            tree: TRoot::new(),
        }
    }

    fn construct_tree(&mut self) {
        construct_tree(&mut self.tree, self.input)
            .map_err(|err| Error::FailedToCreateTurtleStructure(err.to_string()))
            .unwrap();
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

    fn cmp_blank_nodes(a: &BlankNode, b: &BlankNode) -> Ordering {
        a.as_str().cmp(b.as_str())
    }

    fn cmp_subj(a: &&Subject, b: &&Subject) -> Ordering {
        match (a, b) {
            (Subject::NamedNode(_a), Subject::BlankNode(_b)) => Ordering::Greater,
            (Subject::BlankNode(_a), Subject::NamedNode(_b)) => Ordering::Less,
            (Subject::NamedNode(a), Subject::NamedNode(b)) => a.cmp(b),
            (Subject::BlankNode(a), Subject::BlankNode(b)) => Self::cmp_blank_nodes(a, b),
        }
    }

    fn cmp_tsubj(a: &TSubject, b: &TSubject) -> Ordering {
        // match (a, b) {
        //     (Subject::NamedNode(_a), Subject::BlankNode(_b)) => Ordering::Greater,
        //     (Subject::BlankNode(_a), Subject::NamedNode(_b)) => Ordering::Less,
        //     (Subject::NamedNode(a), Subject::NamedNode(b)) => a.cmp(b),
        //     (Subject::BlankNode(a), Subject::BlankNode(b)) => Self::cmp_blank_nodes(a, b),
        // }
        todo!();
    }

    fn cmp_pred(a: &&NamedNode, b: &&NamedNode) -> Ordering {
        a.cmp(b)
    }

    fn cmp_literal(a: &Literal, b: &Literal) -> Ordering {
        let cmp_value = a.value().cmp(b.value());
        if cmp_value != Ordering::Equal {
            return cmp_value;
        }
        let cmp_datatype = a.datatype().cmp(&b.datatype());
        if cmp_datatype != Ordering::Equal {
            return cmp_datatype;
        }
        match (a.language(), b.language()) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_a), None) => Ordering::Less,
            (None, Some(_b)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }

    fn cmp_obj(a: &&Term, b: &&Term) -> Ordering {
        match (a, b) {
            (Term::NamedNode(_a), Term::BlankNode(_b)) => Ordering::Greater,
            (Term::BlankNode(_a), Term::NamedNode(_b)) => Ordering::Less,
            (Term::NamedNode(a), Term::NamedNode(b)) => Self::cmp_pred(&a, &b),
            (Term::BlankNode(a), Term::BlankNode(b)) => Self::cmp_blank_nodes(a, b),
            (Term::NamedNode(_a), Term::Literal(_b)) => Ordering::Greater,
            (Term::Literal(_a), Term::NamedNode(_b)) => Ordering::Less,
            (Term::Literal(a), Term::Literal(b)) => Self::cmp_literal(a, b),
            (Term::BlankNode(_a), Term::Literal(_b)) => Ordering::Greater,
            (Term::Literal(_a), Term::BlankNode(_b)) => Ordering::Less,
        }
    }

    // fn named_node_as_string(&self, named_node: &NamedNode) -> Result<String> {
    //     let iri: &str = named_node.as_str();

    //     if let Some(base_iri) = self.input.base.as_deref() {
    //         if iri.starts_with(base_iri) {
    //             let baseless_iri = &iri[base_iri.len()..];
    //             return Ok(format!("<{baseless_iri}>"));
    //         }
    //     }

    //     let local_name = RE_NAMESPACE_DIVIDER.split(iri).last().unwrap();
    //     let iri: &str = named_node.as_str();
    //     let namespace = &iri[0..(iri.len() - local_name.len())];
    //     Ok(
    //         if let Some(prefix) = self.input.prefixes_inverted.get(namespace) {
    //             format!("{prefix}:{local_name}")
    //         } else {
    //             format!("<{iri}>")
    //         },
    //     )
    // }

    // fn blank_node_as_string(&self, blank_node: &BlankNode) -> Result<String> {
    //     // This prints out the name, e.g. "_:node0"
    //     Ok(blank_node.to_string())
    // }

    fn write_indent<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        for _ in 0..context.indent_level {
            write!(context.output, "{}", self.options.indentation)?;
        }
        Ok(())
    }

    // fn subj_as_string(&self, subj: &Subject) -> Result<String> {
    //     Ok(match subj {
    //         Subject::NamedNode(node) => self.named_node_as_string(node)?,
    //         Subject::BlankNode(node) => self.blank_node_as_string(node)?,
    //     })
    // }

    fn fmt_named_node<W: Write>(
        &self,
        context: &mut Context<W>,
        named_node: &NamedNodeRef<'_>,
    ) -> Result<()> {
        self.write_indent(context)?;

        let iri: &str = named_node.as_str();

        if iri == NN_RDF_TYPE.as_str() {
            write!(context.output, "a")?;
            return Ok(());
        }

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
        writeln!(context.output)?;
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
            context.indent_level += 1;
            for predicates_cont in predicates_conts {
                // writeln!(context.output)?;
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
            self.write_indent(context)?;
            if final_dot {
                writeln!(context.output, ".")?;
            }
            context.indent_level -= 1;
        }
        Ok(())
    }

    // fn term_as_string(&self, term: &Term) -> Result<String> {
    //     Ok(match term {
    //         Term::NamedNode(node) => self.named_node_as_string(node)?,
    //         Term::BlankNode(node) => self.blank_node_as_string(node)?,
    //         Term::Literal(node) => node.to_string(),
    //     })
    // }

    fn fmt_triples<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        let mut sorted_subjects: Vec<_> = self.tree.subjects.iter().collect();
        // sorted_subjects.sort_by(Self::cmp_tsubj);
        for subj_cont in sorted_subjects {
            self.fmt_subj_cont(context, subj_cont)?;
        }
        Ok(())
    }

    fn fmt_doc<W: Write>(&self, context: &mut Context<W>) -> Result<()> {
        self.fmt_base(context)?;

        // let sorted_prefixes = utils::sort_prefixes(self.options, self.input.prefixes)?;
        self.fmt_prefixes(context)?;

        writeln!(context.output)?;

        // self.sort_triples()?;
        self.fmt_triples(context)?;
        Ok(())
    }
}
