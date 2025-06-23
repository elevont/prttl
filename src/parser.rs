// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

use oxrdf::{graph::CanonicalizationAlgorithm, Graph};
use oxttl::TurtleParser;

use thiserror::Error;

use crate::{constants::SUBSTITUTE_BASE, input::Input, options::FormatOptions};

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "We do not support redefinition of prefixes,
which is the case with '{0}'.

For more information, see:
<https://codeberg.org/elevont/prttl/src/branch/main/DesignDecisions.md#prefix-redefinition>"
    )]
    PrefixRedefinition(String),

    #[error(
        "We do not support multiple prefixes for a single namespace. \
Please consider refactoring the input first. \
More info can be found at ...
Conflicting namespaces:
{0:#?}

For more information, see:
<https://codeberg.org/elevont/prttl/src/branch/main/DesignDecisions.md#prefixes-with-equal-namespace>"
    )]
    MultiplePrefixesForNamespace(HashMap<String, Vec<String>>),

    #[error(
        "We do not support handling of comments.
Please consider refactoring.
The reason for that and hints for how to do the refactoring
can be found at <https://codeberg.org/elevont/prttl/src/branch/main/DesignDecisions.md#comments>.

Alternatively, you may choose to `--force` the pretty-printing anyway,
**Which will remove all the Turtle syntax comments in your file!**"
    )]
    Comment,

    #[error(
        "We do not support more then one base IRI defined per file. \
Please consider refactoring the input first.

For more information, see:
<https://codeberg.org/elevont/prttl/src/branch/main/DesignDecisions.md#base-redefinition>"
    )]
    BaseRedefinition,

    #[error(
        "We do not support a prefix ({0}) and a base to cover the same namespace. \
Please consider refactoring the input first.

For more information, see:
<https://codeberg.org/elevont/prttl/src/branch/main/DesignDecisions.md#prefix-vs-base>"
    )]
    PrefixAndBaseShareNamespace(String),

    #[error(transparent)]
    TurtleSyntaxError(#[from] oxttl::TurtleSyntaxError),

    #[error("Failed to parse as base IRI: '{0}'")]
    BaseIri(#[from] oxrdf::IriParseError),
}

fn find_duplicate_values(map: &BTreeMap<String, String>) -> HashMap<String, Vec<String>> {
    let mut value2keys = HashMap::new();
    for (key, value) in map {
        value2keys
            .entry(value)
            .or_insert_with(Vec::default)
            .push(key);
    }
    value2keys
        .into_iter()
        .filter_map(|(value, keys)| {
            if keys.len() > 1 {
                Some((value.to_owned(), keys.into_iter().cloned().collect()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>()
}

/// Parses a given (supposedly) Turtle file content into an [`Input`],
/// which can then be fed into [`crate::formatter::format`].
///
/// # Errors
///
/// - [`Error::TurtleSyntaxError`]
/// - [`Error::BaseRedefinition`]
/// - [`Error::PrefixRedefinition`]
/// - [`Error::MultiplePrefixesForNamespace`]
/// - [`Error::PrefixAndBaseShareNamespace`]
pub fn parse(turtle_str: &[u8], options: &Rc<FormatOptions>) -> Result<Input, Error> {
    let mut graph = Graph::new();

    let mut parser = TurtleParser::new()
        .with_base_iri(SUBSTITUTE_BASE)?
        .low_level();
    if let Some(parse_res) = parser.parse_next() {
        parse_res?;
    }
    parser.extend_from_slice(turtle_str.as_ref());
    parser.end();
    let mut base = None;
    let mut prefixes = HashMap::new();
    let mut seen_subjects = HashSet::new();
    let mut subjects_in_order = Vec::new();
    let mut seen_bn_objects = HashSet::new();
    let mut bn_objects_input_order = Vec::new();
    while let Some(triple_res) = parser.parse_next() {
        let triple = triple_res?;

        if seen_subjects.insert(triple.subject.clone()) {
            subjects_in_order.push(triple.subject.clone());
        }

        // NOTE We do the ref and then into_owned again,
        //      because this way we do not clone all the (potentially huge) objects.
        if let TermRef::BlankNode(bn) = triple.object.as_ref() {
            if seen_bn_objects.insert(bn.into_owned()) {
                bn_objects_input_order.push(bn.into_owned());
            }
        }

        graph.insert(&triple);

        // validate & store base
        if let Some(cur_base) = parser.base_iri() {
            if let Some(base_val) = base {
                if base_val != cur_base {
                    return Err(Error::BaseRedefinition);
                }
            }
            base = Some(cur_base.to_owned());
        }

        // validate & store prefixes
        for cur_prefix in parser.prefixes() {
            if let Some(cur_val) = prefixes.get(cur_prefix.0) {
                if cur_val != cur_prefix.1 {
                    return Err(Error::PrefixRedefinition(cur_prefix.0.to_owned()));
                }
            } else {
                prefixes.insert(cur_prefix.0.to_owned(), cur_prefix.1.to_owned());
            }
        }
    }
    // handle case of Turtle syntax comments found in the source
    if parser.seen_comment() {
        if options.force {
            tracing::info!(
                "Even though comments were found in the input,
we continue formatting (which removes all of them),
because the 'force' option was specified!"
            );
        } else {
            return Err(Error::Comment);
        }
    }
    tracing::debug!("Low level parsing went ok!");
    if options.canonicalize {
        graph.canonicalize(CanonicalizationAlgorithm::Unstable);
    }

    let prefixes_sorted = BTreeMap::from_iter(prefixes.clone());
    let prefixes_inverted: HashMap<String, String> =
        prefixes.into_iter().map(|(k, v)| (v, k)).collect();
    if prefixes_sorted.len() > prefixes_inverted.len() {
        let duplicate_prefixes = find_duplicate_values(&prefixes_sorted);
        return Err(Error::MultiplePrefixesForNamespace(duplicate_prefixes));
    }

    if let Some(base_val) = &base {
        if let Some(prefix) = prefixes_inverted.get(base_val) {
            return Err(Error::PrefixAndBaseShareNamespace(prefix.to_owned()));
        }
    }

    let input = Input {
        base,
        prefixes: prefixes_sorted,
        prefixes_inverted,
        subjects_in_order,
        bn_objects_input_order,
        graph,
    };

    Ok(input)
}
