// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::collections::HashMap;

use oxrdf::Graph;
use oxrdf::NamedOrBlankNodeRef;

pub struct Input {
    pub base: Option<String>,
    pub prefixes: BTreeMap<String, String>,
    pub prefixes_inverted: HashMap<String, String>,
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
