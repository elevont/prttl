// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashMap};

use oxrdf::{graph::CanonicalizationAlgorithm, Graph};
use oxttl::TurtleParser;

use thiserror::Error;

use crate::{input::Input, options::FormatOptions};

#[derive(Error, Debug)]
pub enum Error {
    #[error("We do not support redefinition of prefixes, which is the case with {0}")]
    PrefixRedefinition(String),

    #[error("We do not support more then one base IRI defined per file")]
    MultipleBases,

    #[error(transparent)]
    TurtleSyntaxError(#[from] oxttl::TurtleSyntaxError),
}

pub fn parse(turtle_str: &[u8], options: &FormatOptions) -> Result<Input, Error> {
    let mut graph = Graph::new();

    let mut parser = TurtleParser::new().low_level();
    parser.extend_from_slice(turtle_str.as_ref());
    let mut base = None;
    let mut prefixes = HashMap::new();
    while let Some(triple_res) = parser.parse_next() {
        let triple = triple_res?;
        graph.insert(&triple);

        // validate & store base
        if let Some(cur_base) = parser.base_iri() {
            if let Some(base_val) = base {
                if base_val != cur_base {
                    return Err(Error::MultipleBases);
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
    if options.canonicalize {
        graph.canonicalize(CanonicalizationAlgorithm::Unstable);
    }

    let prefixes_sorted = BTreeMap::from_iter(prefixes.clone());
    let prefixes_inverted = prefixes.into_iter().map(|(k, v)| (v, k)).collect();

    let input = Input {
        base,
        prefixes: prefixes_sorted,
        prefixes_inverted,
        graph,
    };

    Ok(input)
}
