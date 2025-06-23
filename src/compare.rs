// SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ast::{
        SortingContext, TBlankNode, TBlankNodeRef, TCollection, TCollectionRef, TLiteralRef,
        TNamedNode, TObject, TPredicateCont, TSubject, TSubjectCont, TTriple,
    },
    vocab::prtr,
};
use oxrdf::{vocab::rdf, BlankNodeRef, NamedOrBlankNodeRef, TermRef};
use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
};

#[must_use]
pub fn named_nodes<'graph>(
    _context: &SortingContext<'graph>,
    a: &TNamedNode<'graph>,
    b: &TNamedNode<'graph>,
) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }

    let a_type_num: u8 = a.into();
    let b_type_num: u8 = b.into();
    let type_cmp = a_type_num.cmp(&b_type_num);
    if type_cmp == Ordering::Equal {
        a.cmp(b)
    } else {
        type_cmp
    }
}

#[must_use]
pub const fn blank_node_refs_by_input_order(
    _context: &SortingContext<'_>,
    _a: &BlankNodeRef,
    _b: &BlankNodeRef,
) -> Ordering {
    // By label -> Don't do this!
    // a.as_str().cmp(b.as_str())

    // Same order as in the input
    Ordering::Equal
}

#[must_use]
pub fn blank_node_refs<'graph>(
    context: &SortingContext<'graph>,
    a: &BlankNodeRef<'graph>,
    b: &BlankNodeRef<'graph>,
) -> Ordering {
    if context.options.prtr_sorting {
        blank_node_refs_with_prtr(context, a, b)
    } else {
        blank_node_refs_by_input_order(context, a, b)
    }
}

#[must_use]
fn fetch_prtr_sorting_id<'graph>(
    context: &SortingContext<'graph>,
    bn: &BlankNodeRef<'graph>,
) -> Option<u32> {
    match context.bn_sorting_ids.borrow_mut().entry(*bn) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let sorting_id_opt = context
                .graph
                .object_for_subject_predicate(
                    NamedOrBlankNodeRef::BlankNode(*bn),
                    *prtr::SORTING_ID,
                )
                .and_then(|sorting_id_term| {
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
                            .ok()
                    } else {
                        None
                    }
                });

            entry.insert(sorting_id_opt);
            sorting_id_opt
        }
    }
}

#[must_use]
pub fn blank_node_refs_with_prtr<'graph>(
    context: &SortingContext<'graph>,
    a: &BlankNodeRef<'graph>,
    b: &BlankNodeRef<'graph>,
) -> Ordering {
    let a_sorting_id_opt = fetch_prtr_sorting_id(context, a);
    let b_sorting_id_opt = fetch_prtr_sorting_id(context, b);

    match (a_sorting_id_opt, b_sorting_id_opt) {
        (Some(a_sorting_id), Some(b_sorting_id)) => a_sorting_id.cmp(&b_sorting_id),
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => blank_node_refs_by_input_order(context, a, b),
    }
}

#[must_use]
pub fn blank_node_refs_by_label<'graph>(
    _context: &SortingContext<'graph>,
    a: &BlankNodeRef<'graph>,
    b: &BlankNodeRef<'graph>,
) -> Ordering {
    a.as_str().cmp(b.as_str())
}

#[must_use]
pub fn t_blank_nodes<'graph>(
    context: &SortingContext<'graph>,
    a: &TBlankNode<'graph>,
    b: &TBlankNode<'graph>,
) -> Ordering {
    blank_node_refs(context, &a.node.0, &b.node.0)
}

#[must_use]
fn t_blank_nodes_by_object_input_order<'graph>(
    context: &SortingContext<'graph>,
    a: &TBlankNode<'graph>,
    b: &TBlankNode<'graph>,
) -> Ordering {
    let a_idx = context
        .bn_objects_input_order
        .get(&a.node.0.into_owned())
        .expect("This should always contain all the blank nodes appearing as objects in the graph");
    let b_idx = context
        .bn_objects_input_order
        .get(&b.node.0.into_owned())
        .expect("This should always contain all the blank nodes appearing as objects in the graph");
    a_idx.cmp(b_idx)
}

#[must_use]
pub fn t_collection_refs<'graph>(
    context: &SortingContext<'graph>,
    a: &TCollectionRef<'graph>,
    b: &TCollectionRef<'graph>,
) -> Ordering {
    let mut b_iter = b.rest.iter();
    for a_item in &a.rest {
        if let Some(b_item) = b_iter.next() {
            let cmp_item = t_obj(context, a_item, b_item);
            if cmp_item != Ordering::Equal {
                return cmp_item;
            }
        } else {
            return Ordering::Greater;
        }
    }
    if b_iter.next().is_some() {
        return Ordering::Less;
    }
    t_blank_nodes(context, &a.node, &b.node)
}

#[must_use]
pub fn t_collections<'graph>(
    context: &SortingContext<'graph>,
    a: &TCollection<'graph>,
    b: &TCollection<'graph>,
) -> Ordering {
    match (a, b) {
        (TCollection::Empty, TCollection::Empty) => Ordering::Equal,
        (TCollection::Empty, TCollection::WithContent(_)) => Ordering::Less,
        (TCollection::WithContent(_), TCollection::Empty) => Ordering::Greater,
        (TCollection::WithContent(a), TCollection::WithContent(b)) => {
            t_collection_refs(context, a, b)
        }
    }
}

#[must_use]
pub fn triples<'graph>(
    context: &SortingContext<'graph>,
    a: &TTriple<'graph>,
    b: &TTriple<'graph>,
) -> Ordering {
    let cmp_subj = t_subj(context, &a.0, &b.0);
    if cmp_subj != Ordering::Equal {
        return cmp_subj;
    }
    let cmp_pred = pred_ref(context, &a.1, &b.1);
    if cmp_pred != Ordering::Equal {
        return cmp_pred;
    }
    t_obj(context, &a.2, &b.2)
}

#[must_use]
fn extract_topmost_sorting_id_by_types<'graph, S: ::std::hash::BuildHasher>(
    context: &SortingContext<'graph>,
    subject_type_order: &HashMap<String, usize, S>,
    nn: &TNamedNode<'graph>,
) -> Option<usize> {
    let mut topmost_sorting_id = None;
    let types = context
        .graph
        .objects_for_subject_predicate(*nn.as_named_node_ref(), rdf::TYPE)
        .collect::<Vec<_>>();
    for typ in types {
        if let TermRef::NamedNode(typ_nn) = typ {
            if let Some(cur_sorting_id) = subject_type_order.get(typ_nn.as_str()) {
                if let Some(best) = topmost_sorting_id {
                    if *cur_sorting_id > best {
                        continue;
                    }
                }
                topmost_sorting_id = Some(*cur_sorting_id);
            }
        }
    }

    topmost_sorting_id
}

#[must_use]
pub fn t_subj<'graph>(
    context: &SortingContext<'graph>,
    a: &TSubject<'graph>,
    b: &TSubject<'graph>,
) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    match (a, b) {
        (TSubject::NamedNode(a), TSubject::NamedNode(b)) => {
            if let Some(subject_type_order) = context.subject_type_order.as_ref() {
                let a_best_sorting_id =
                    extract_topmost_sorting_id_by_types(context, subject_type_order, a);
                let b_best_sorting_id =
                    extract_topmost_sorting_id_by_types(context, subject_type_order, b);
                match (a_best_sorting_id, b_best_sorting_id) {
                    (Some(a), Some(b)) => {
                        let special_order = a.cmp(&b);
                        if special_order != Ordering::Equal {
                            return special_order;
                        }
                    }
                    (Some(_a), None) => return Ordering::Less,
                    (None, Some(_b)) => return Ordering::Greater,
                    (None, None) => (),
                }
            }
            named_nodes(context, a, b)
        }
        (
            TSubject::BlankNodeLabel(TBlankNodeRef(a)),
            TSubject::BlankNodeLabel(TBlankNodeRef(b)),
        ) => blank_node_refs(context, a, b),
        (TSubject::BlankNodeAnonymous(a), TSubject::BlankNodeAnonymous(b)) => {
            t_blank_nodes(context, a, b)
        }
        (TSubject::Collection(a), TSubject::Collection(b)) => t_collections(context, a, b),
        (TSubject::Triple(a), TSubject::Triple(b)) => triples(context, a, b),
        (a, b) => {
            let a_type_num: u8 = a.into();
            let b_type_num: u8 = b.into();
            a_type_num.cmp(&b_type_num)
        }
    }
}

#[must_use]
pub fn t_subj_cont<'graph>(
    context: &SortingContext<'graph>,
    a: &TSubjectCont<'graph>,
    b: &TSubjectCont<'graph>,
) -> Ordering {
    t_subj(context, &a.subject, &b.subject)
}

#[must_use]
pub fn pred_ref<'graph>(
    context: &SortingContext<'graph>,
    a: &TNamedNode<'graph>,
    b: &TNamedNode<'graph>,
) -> Ordering {
    match (
        context.predicate_order.get(a.as_named_node_ref().as_str()),
        context.predicate_order.get(b.as_named_node_ref().as_str()),
    ) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_a), None) => Ordering::Less,
        (None, Some(_b)) => Ordering::Greater,
        (None, None) => named_nodes(context, a, b),
    }
}

#[must_use]
pub fn t_pred_cont<'graph>(
    context: &SortingContext<'graph>,
    a: &TPredicateCont<'graph>,
    b: &TPredicateCont<'graph>,
) -> Ordering {
    pred_ref(context, &a.predicate, &b.predicate)
}

#[must_use]
pub fn t_obj<'graph>(
    context: &SortingContext<'graph>,
    a: &TObject<'graph>,
    b: &TObject<'graph>,
) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    match (a, b) {
        (TObject::NamedNode(a), TObject::NamedNode(b)) => named_nodes(context, a, b),
        (TObject::BlankNodeLabel(TBlankNodeRef(a)), TObject::BlankNodeLabel(TBlankNodeRef(b))) => {
            blank_node_refs(context, a, b)
        }
        (TObject::BlankNodeAnonymous(a), TObject::BlankNodeAnonymous(b)) => {
            let bn_cmp = t_blank_nodes(context, a, b);
            if bn_cmp != Ordering::Equal {
                return bn_cmp;
            }
            t_blank_nodes_by_object_input_order(context, a, b)
        }
        (TObject::Collection(a), TObject::Collection(b)) => t_collections(context, a, b),
        (TObject::Literal(a), TObject::Literal(b)) => literals(context, a, b),
        (TObject::Triple(a), TObject::Triple(b)) => triples(context, a, b),
        (a, b) => {
            let a_type_num: u8 = a.into();
            let b_type_num: u8 = b.into();
            a_type_num.cmp(&b_type_num)
        }
    }
}

#[must_use]
pub fn literals<'graph>(
    context: &SortingContext<'graph>,
    a: &TLiteralRef<'graph>,
    b: &TLiteralRef<'graph>,
) -> Ordering {
    // 1. by *nice* data-type
    let nice_dt_cmp = match (a.1.as_ref(), b.1.as_ref()) {
        (Some(a), Some(b)) => named_nodes(context, a, b),
        (Some(_a), None) => Ordering::Less,
        (None, Some(_b)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    if nice_dt_cmp != Ordering::Equal {
        return nice_dt_cmp;
    }

    // 2. by regular data-type
    let cmp_datatype = a.0.datatype().cmp(&b.0.datatype());
    if cmp_datatype != Ordering::Equal {
        return cmp_datatype;
    }

    // 3. by language
    let language_cmp = match (a.0.language(), b.0.language()) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_a), None) => Ordering::Less,
        (None, Some(_b)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    if language_cmp != Ordering::Equal {
        return cmp_datatype;
    }

    // 4. by value
    a.0.value().cmp(b.0.value())
}
