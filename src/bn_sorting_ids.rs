// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use oxrdf::BlankNodeRef;
use oxrdf::NamedOrBlankNodeRef;
use oxrdf::TermRef;
use oxrdf::TripleRef;
use thiserror::Error;

use crate::input::Input;
use crate::options::FormatOptions;
use crate::vocab::prtr;

type SortingId = u32;

/// A cache for blank node sorting ids (`prtr::sortingId`).
/// We use a cache for performance reasons.
pub struct Cache<'graph> {
    options: Rc<FormatOptions>,
    input: &'graph Input,
    /// The sorting ID associated with each blank node.
    bn_sorting_ids: HashMap<BlankNodeRef<'graph>, Option<SortingId>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(
        "Tried to generate IDs for blank nodes without one,
but already assigned sorting IDs are not in order"
    )]
    IdsNotInOrder,
}

impl<'graph> Cache<'graph> {
    #[must_use]
    pub fn new<'new>(input: &'new Input, options: Rc<FormatOptions>) -> Self
    where
        'new: 'graph,
    {
        Self {
            input,
            options,
            bn_sorting_ids: HashMap::new(),
        }
    }

    #[must_use]
    pub fn fetch_prtr_sorting_id<'sorting>(&mut self, bn: &BlankNodeRef<'graph>) -> Option<u32>
    where
        'graph: 'sorting,
    {
        match self.bn_sorting_ids./*borrow_mut().*/entry(*bn) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let sorting_ids = self
                    .input
                    .graph
                    .objects_for_subject_predicate(
                        NamedOrBlankNodeRef::BlankNode(*bn),
                        *prtr::SORTING_ID,
                    )
                    .collect::<Vec<_>>();

                let sorting_id_opt = if sorting_ids.is_empty() {
                    None
                } else if sorting_ids.len() > 1 {
                    tracing::error!(
                        "Multiple prtr:sortingId values for blank node {bn}. Please reduce to one.",
                    );
                    std::process::exit(2);
                    None
                } else {
                    sorting_ids.first().and_then(|sorting_id_term| {
                        if let TermRef::Literal(sorting_id_literal) = sorting_id_term {
                            sorting_id_literal
                                .value()
                                .parse()
                                .map_err(|err| {
                                    tracing::warn!(
                                        "Failed to parse prtr:sortingId value ('{}') as u32: {err}",
                                        sorting_id_literal.value()
                                    );
                                    err
                                })
                                .ok() // TODO Maybe panic! instead?
                        } else {
                            None
                        }
                    })
                };

                entry.insert(sorting_id_opt);
                sorting_id_opt
            }
        }
    }

    fn collect_non_id_able_bn_subjects<'tree>(
        col_involved_triples: &Vec<TripleRef<'graph>>,
    ) -> HashSet<NamedOrBlankNodeRef<'graph>>
    where
        'graph: 'tree,
    {
        col_involved_triples
            .iter()
            .map(|triple| triple.subject)
            .collect::<HashSet<_>>()
    }

    fn collect_bns_and_their_sorting_ids<'tree>(
        &mut self,
        col_involved_triples: &Vec<TripleRef<'graph>>,
    ) -> (Vec<(BlankNodeRef<'graph>, Option<u32>)>, bool, bool)
    where
        'graph: 'tree,
    {
        let non_id_able_subjects = Self::collect_non_id_able_bn_subjects(col_involved_triples);

        let mut bn_subj_in_order = vec![];
        // Collect all blank nodes that need a sorting ID
        // and their sorting ID, if they already have one (else: None),
        // in the order the blank nodes appear in the input.
        let mut used_ids = HashSet::new();
        let mut biggest_id_so_far = None;
        let mut ids_in_order = true;
        for subj in &self.input.subjects_in_order {
            if let NamedOrBlankNodeRef::BlankNode(bn) = subj.as_ref() {
                if !non_id_able_subjects.contains(&subj.as_ref()) {
                    let sorting_id = self.fetch_prtr_sorting_id(&bn);
                    if let Some(id) = sorting_id {
                        used_ids.insert(id);
                        if let Some(biggest_id_so_far_val) = biggest_id_so_far {
                            if id > biggest_id_so_far_val {
                                biggest_id_so_far = Some(id);
                            } else {
                                ids_in_order = false;
                            }
                        } else {
                            biggest_id_so_far = Some(id);
                        }
                    }
                    bn_subj_in_order.push((bn, sorting_id));
                }
            }
        }

        (bn_subj_in_order, ids_in_order, !used_ids.is_empty())
    }

    /// Extracts all blank nodes that could/should have a `prtr:sortingId`
    /// in the order in which they appear in the input.
    ///
    /// # Errors
    ///
    /// TODO WRONG! Never fails (Infallible).
    pub fn ensure_sorting_ids_assigned<'tree>(
        &mut self,
        col_involved_triples: &Vec<TripleRef<'graph>>,
    ) -> Result<(), Error>
    where
        'graph: 'tree,
    {
        let (mut bn_subj_in_order, ids_in_order, used_ids) =
            self.collect_bns_and_their_sorting_ids(col_involved_triples);

        if bn_subj_in_order.len() == self.input.subjects_in_order.len() {
            return Ok(());
        }

        if used_ids {
            // Assign sorting IDs to the remaining blank nodes.
            // If those that already have IDs are
            // let mut un_ided_bns = vec![];
            if self.options.prioritize_input_order {
                if !ids_in_order {
                    return Err(Error::IdsNotInOrder);
                }
                let mut next_id = 0;
                let mut assigned = 0;
                for bn_srtid in &mut bn_subj_in_order {
                    let do_assign = bn_srtid.1.is_none_or(|sorting_id| {
                        if next_id > sorting_id {
                            true
                        } else {
                            next_id = sorting_id + 1;
                            false
                        }
                    });
                    if do_assign {
                        bn_srtid.1 = Some(next_id);
                        next_id += 1;
                        assigned += 1;
                    }
                }
                if assigned == bn_subj_in_order.len() {
                    // all were assigned or reassigned,
                    // so we re-assign all again, but this time
                    // with 99 IDs in-between each consecutive blank nodes.
                    // This ensures that inserting new blank nodes in the future
                    // will unlikely result in the need for re-assigning IDs
                    assign_all_ids(&mut bn_subj_in_order);
                }
            } else {
                let mut next_free_id = 0;
                // holds blank nodes without sorting ID
                // in-between two blank-nodes *with* sorting IDs,
                // or the start or end of the list of blank-nodes respectively.
                let mut unassigned_streak: LinkedList<&mut (BlankNodeRef<'_>, Option<u32>)> =
                    LinkedList::new();
                for bn_srtid in &mut bn_subj_in_order {
                    if let Some(sorting_id) = bn_srtid.1 {
                        while next_free_id < sorting_id {
                            if let Some(unassigned_bn) = unassigned_streak.pop_front() {
                                unassigned_bn.1 = Some(next_free_id);
                                next_free_id += 1;
                            } else {
                                break;
                            }
                        }
                        next_free_id = sorting_id + 1;
                    } else {
                        unassigned_streak.push_back(bn_srtid);
                    }
                }
                while let Some(unassigned_bn) = unassigned_streak.pop_front() {
                    unassigned_bn.1 = Some(next_free_id);
                    next_free_id += 1;
                }
            }
        } else {
            assign_all_ids(&mut bn_subj_in_order);
        }

        Ok(())
    }
}

fn assign_all_ids(bn_subj_in_order: &mut [(BlankNodeRef<'_>, Option<u32>)]) {
    let mut next_id = 1000;
    for bn_srtid in bn_subj_in_order.iter_mut() {
        bn_srtid.1 = Some(next_id);
        next_id += 100;
    }
}
