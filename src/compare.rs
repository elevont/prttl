// SPDX-FileCopyrightText: 2022 Helsing GmbH
//
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ast::{
        SortingContext, TBlankNode, TBlankNodeRef, TCollection, TCollectionRef, TLiteralRef,
        TObject, TPredicateCont, TSubject, TSubjectCont, TTriple,
    },
    vocab::prtyr,
};
use oxrdf::{vocab::rdf, BlankNode, BlankNodeRef, LiteralRef, NamedNodeRef, SubjectRef, TermRef};
use std::cmp::Ordering;

#[must_use]
pub fn named_nodes<'graph>(a: &NamedNodeRef<'graph>, b: &NamedNodeRef<'graph>) -> Ordering {
    if a == b {
        Ordering::Equal
    } else if *a == rdf::TYPE {
        Ordering::Less
    } else if *b == rdf::TYPE {
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
pub fn blank_node_refs<'graph>(a: &BlankNodeRef<'graph>, b: &BlankNodeRef<'graph>) -> Ordering {
    a.as_str().cmp(b.as_str())
}

#[must_use]
pub fn t_blank_nodes<'graph>(
    context: &SortingContext<'graph>,
    a: &TBlankNode<'graph>,
    b: &TBlankNode<'graph>,
) -> Ordering {
    if context.options.prtyr_sorting {
        t_blank_nodes_with_prtyr(context, a, b)
    } else {
        t_blank_nodes_by_label(context, a, b)
    }
}

#[must_use]
fn t_blank_nodes_by_label<'graph>(
    context: &SortingContext<'graph>,
    a: &TBlankNode<'graph>,
    b: &TBlankNode<'graph>,
) -> Ordering {
    let a_bn = a.node.0.as_str();
    let b_bn = b.node.0.as_str();
    a_bn.cmp(b_bn)
}

#[must_use]
fn t_blank_nodes_with_prtyr<'graph>(
    context: &SortingContext<'graph>,
    a: &TBlankNode<'graph>,
    b: &TBlankNode<'graph>,
) -> Ordering {
    let graph = context.graph;

    let a_sorting_id_opt =
        graph.object_for_subject_predicate(SubjectRef::BlankNode(a.node.0), *prtyr::SORTING_ID);
    let b_sorting_id_opt =
        graph.object_for_subject_predicate(SubjectRef::BlankNode(b.node.0), *prtyr::SORTING_ID);

    match (a_sorting_id_opt, b_sorting_id_opt) {
        (Some(TermRef::Literal(a_sorting_id)), Some(TermRef::Literal(b_sorting_id))) => {
            let a_int: u32 = a_sorting_id
                .value()
                .parse()
                .expect("Failed to parse prtyr:sortingId value as u32");
            let b_int: u32 = b_sorting_id
                .value()
                .parse()
                .expect("Failed to parse prtyr:sortingId value as u32");
            a_int.cmp(&b_int)
        }
        (Some(_), Some(_)) => panic!("At least one prtyr:sortingId value is not a literal"),
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => t_blank_nodes_by_label(context, a, b),
    }
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
    let cmp_pred = pred_ref(context, &a.1 .0, &b.1 .0);
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
        (TSubject::NamedNode(a), TSubject::NamedNode(b)) => named_nodes(a, b),
        (
            TSubject::BlankNodeLabel(TBlankNodeRef(a)),
            TSubject::BlankNodeLabel(TBlankNodeRef(b)),
        ) => blank_node_refs(a, b),
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
    a: &NamedNodeRef<'graph>,
    b: &NamedNodeRef<'graph>,
) -> Ordering {
    named_nodes(a, b)
}

#[must_use]
pub fn t_pred_cont<'graph>(
    context: &SortingContext<'graph>,
    a: &TPredicateCont<'graph>,
    b: &TPredicateCont<'graph>,
) -> Ordering {
    pred_ref(context, &a.predicate.0, &b.predicate.0)
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
        (TObject::NamedNode(a), TObject::NamedNode(b)) => named_nodes(a, b),
        (TObject::BlankNodeLabel(TBlankNodeRef(a)), TObject::BlankNodeLabel(TBlankNodeRef(b))) => {
            blank_node_refs(a, b)
        }
        (TObject::BlankNodeAnonymous(a), TObject::BlankNodeAnonymous(b)) => {
            t_blank_nodes(context, a, b)
        }
        (TObject::Collection(a), TObject::Collection(b)) => t_collections(context, a, b),
        (TObject::Literal(TLiteralRef(a)), TObject::Literal(TLiteralRef(b))) => literals(a, b),
        (TObject::Triple(a), TObject::Triple(b)) => triples(context, a, b),
        (a, b) => {
            let a_type_num: u8 = a.into();
            let b_type_num: u8 = b.into();
            a_type_num.cmp(&b_type_num)
        }
    }
}

#[must_use]
pub fn literals<'graph>(a: &LiteralRef<'graph>, b: &LiteralRef<'graph>) -> Ordering {
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
