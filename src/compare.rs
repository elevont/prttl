// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ast::{
        SortingContext, TBlankNode, TBlankNodeRef, TCollection, TCollectionRef, TLiteralRef,
        TNamedNode, TObject, TPredicateCont, TSubject, TSubjectCont, TTriple,
    },
    vocab::prtyr,
};
use oxrdf::{vocab::rdf, BlankNode, BlankNodeRef, SubjectRef, TermRef};
use std::cmp::Ordering;

#[must_use]
pub fn named_nodes<'graph>(
    _context: &SortingContext<'graph>,
    a: &TNamedNode<'graph>,
    b: &TNamedNode<'graph>,
) -> Ordering {
    if a == b {
        Ordering::Equal
    } else if *a.as_named_node_ref() == rdf::TYPE {
        Ordering::Less
    } else if *b.as_named_node_ref() == rdf::TYPE {
        Ordering::Greater
    } else {
        a.cmp(b)
    }
}

#[must_use]
pub fn blank_nodes(a: &BlankNode, b: &BlankNode) -> Ordering {
    a.as_str().cmp(b.as_str())
}

#[must_use]
pub fn blank_node_refs<'graph>(
    context: &SortingContext<'graph>,
    a: &BlankNodeRef<'graph>,
    b: &BlankNodeRef<'graph>,
) -> Ordering {
    if context.options.prtyr_sorting {
        blank_node_refs_with_prtyr(context, a, b)
    } else {
        blank_node_refs_by_label(context, a, b)
    }
}

#[must_use]
fn fetch_prtyr_sorting_id<'graph>(
    context: &SortingContext<'graph>,
    bn: &BlankNodeRef<'graph>,
) -> Option<u32> {


    context.bn_sorting_ids.borrow().get(bn).map_or_else(
        || {
            let sorting_id_opt = context
                .graph
                .object_for_subject_predicate(SubjectRef::BlankNode(*bn), *prtyr::SORTING_ID)
                .and_then(|sorting_id_term| {
                    if let TermRef::Literal(sorting_id_literal) = sorting_id_term {
                        sorting_id_literal
                            .value()
                            .parse()
                            .map_err(|err| {
                                tracing::warn!(
                                    "Failed to parse prtyr:sortingId value ('{}') as u32: {err}",
                                    sorting_id_literal.value()
                                );
                                err
                            })
                            .ok()
                    } else {
                        None
                    }
                });

            context
                .bn_sorting_ids
                .borrow_mut()
                .insert(*bn, sorting_id_opt);
            sorting_id_opt
        },
        |id_opt| *id_opt,
    )
}

#[must_use]
pub fn blank_node_refs_with_prtyr<'graph>(
    context: &SortingContext<'graph>,
    a: &BlankNodeRef<'graph>,
    b: &BlankNodeRef<'graph>,
) -> Ordering {
    let a_sorting_id_opt = fetch_prtyr_sorting_id(context, a);
    let b_sorting_id_opt = fetch_prtyr_sorting_id(context, b);

    match (a_sorting_id_opt, b_sorting_id_opt) {
        (Some(a_sorting_id), Some(b_sorting_id)) => a_sorting_id.cmp(&b_sorting_id),
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => blank_node_refs_by_label(context, a, b),
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
pub fn t_subj<'graph>(
    context: &SortingContext<'graph>,
    a: &TSubject<'graph>,
    b: &TSubject<'graph>,
) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    match (a, b) {
        (TSubject::NamedNode(a), TSubject::NamedNode(b)) => named_nodes(context, a, b),
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
    named_nodes(context, a, b)
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
            t_blank_nodes(context, a, b)
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
    let cmp_value = a.0.value().cmp(b.0.value());
    if cmp_value != Ordering::Equal {
        return cmp_value;
    }
    let nice_dt_cmp = match (a.1.as_ref(), b.1.as_ref()) {
        (Some(a), Some(b)) => named_nodes(context, a, b),
        (Some(_a), None) => Ordering::Less,
        (None, Some(_b)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    if nice_dt_cmp != Ordering::Equal {
        return nice_dt_cmp;
    }
    let cmp_datatype = a.0.datatype().cmp(&b.0.datatype());
    if cmp_datatype != Ordering::Equal {
        return cmp_datatype;
    }
    match (a.0.language(), b.0.language()) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_a), None) => Ordering::Less,
        (None, Some(_b)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
