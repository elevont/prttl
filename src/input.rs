// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::collections::HashMap;

use oxrdf::BlankNode;
use oxrdf::Graph;
use oxrdf::NamedOrBlankNode;
use oxrdf::NamedOrBlankNodeRef;

pub struct Input {
    pub base: Option<String>,
    // Prefix to namespace mapping
    pub prefixes: BTreeMap<String, String>,
    // Namespace to prefix mapping
    pub prefixes_inverted: HashMap<String, String>,
    // Subjects in the order they (first) appear in the input
    pub subjects_in_order: Vec<NamedOrBlankNode>,
    // Blank node objects in the order they (first) appear in the input
    pub bn_objects_input_order: Vec<BlankNode>,
    pub graph: Graph,
}

impl Input {
    #[must_use]
    pub fn extract_subjects(&self) -> Vec<NamedOrBlankNodeRef<'_>> {
        let mut subjects = vec![];
        for triple in &self.graph {
            subjects.push(triple.subject);
        }
        subjects
    }
}
