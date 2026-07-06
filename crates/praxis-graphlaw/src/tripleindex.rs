use crate::{Binding, Encoder, Parser, Term, Triple, TripleStore, VarOrTerm};
use either::*;
use std::collections::HashMap;
use std::iter;
use std::iter::empty;
use std::rc::Rc;

pub struct TripleIndex {
    pub triples: Vec<Triple>,
    pub spo: HashMap<usize, HashMap<usize, Vec<(usize, usize, Option<Term>)>>>,
    pub pos: HashMap<usize, HashMap<usize, Vec<(usize, usize, Option<Term>)>>>,
    pub osp: HashMap<usize, HashMap<usize, Vec<(usize, usize, Option<Term>)>>>,
    counter: usize,
}

impl TripleIndex {
    pub fn len(&self) -> usize {
        self.triples.len()
    }
    pub fn get(&self, index: usize) -> Option<&Triple> {
        self.triples.get(index)
    }
    pub fn new() -> TripleIndex {
        TripleIndex {
            triples: Vec::new(),
            spo: HashMap::new(),
            pos: HashMap::new(),
            osp: HashMap::new(),
            counter: 0,
        }
    }
    /// Add a triple via Rc, avoiding duplicates.
    pub(crate) fn add_ref(&mut self, triple: Rc<Triple>) {
        if !self.contains(triple.as_ref()) {
            self.add(triple.as_ref().clone());
        }
    }
    pub fn remove_ref(&mut self, triple: &Triple) {
        //remove spo
        if self.spo.contains_key(&triple.s.to_encoded())
            && self
                .spo
                .get(&triple.s.to_encoded())
                .unwrap()
                .contains_key(&triple.p.to_encoded())
        {
            let spo_values = self
                .spo
                .get_mut(&triple.s.to_encoded())
                .unwrap()
                .get_mut(&triple.p.to_encoded())
                .unwrap();
            spo_values.retain(|(val, counter, _)| *val != triple.o.to_encoded());
        }
        //remove pos
        if self.pos.contains_key(&triple.p.to_encoded())
            && self
                .pos
                .get(&triple.p.to_encoded())
                .unwrap()
                .contains_key(&triple.o.to_encoded())
        {
            let values = self
                .pos
                .get_mut(&triple.p.to_encoded())
                .unwrap()
                .get_mut(&triple.o.to_encoded())
                .unwrap();
            values.retain(|(val, counter, _)| *val != triple.s.to_encoded());
        }
        // remove osp
        if self.osp.contains_key(&triple.o.to_encoded())
            && self
                .osp
                .get(&triple.o.to_encoded())
                .unwrap()
                .contains_key(&triple.s.to_encoded())
        {
            let values = self
                .osp
                .get_mut(&triple.o.to_encoded())
                .unwrap()
                .get_mut(&triple.s.to_encoded())
                .unwrap();
            values.retain(|(val, counter, _)| *val != triple.p.to_encoded());
        }
        self.triples.retain(|t| *t != *triple);
        self.counter -= 1;
    }
    pub fn add(&mut self, triple: Triple) {
        if !self.spo.contains_key(&triple.s.to_encoded()) {
            self.spo.insert(triple.s.to_encoded(), HashMap::new());
        }
        if !self
            .spo
            .get(&triple.s.to_encoded())
            .unwrap()
            .contains_key(&triple.p.to_encoded())
        {
            self.spo
                .get_mut(&triple.s.to_encoded())
                .unwrap()
                .insert(triple.p.to_encoded(), Vec::new());
        }
        self.spo
            .get_mut(&triple.s.to_encoded())
            .unwrap()
            .get_mut(&triple.p.to_encoded())
            .unwrap()
            .push((
                triple.o.to_encoded(),
                self.counter,
                triple.g.clone().map(|g| g.as_term().clone()),
            ));
        //pos
        if !self.pos.contains_key(&triple.p.to_encoded()) {
            self.pos.insert(triple.p.to_encoded(), HashMap::new());
        }
        if !self
            .pos
            .get(&triple.p.to_encoded())
            .unwrap()
            .contains_key(&triple.o.to_encoded())
        {
            self.pos
                .get_mut(&triple.p.to_encoded())
                .unwrap()
                .insert(triple.o.to_encoded(), Vec::new());
        }
        self.pos
            .get_mut(&triple.p.to_encoded())
            .unwrap()
            .get_mut(&triple.o.to_encoded())
            .unwrap()
            .push((
                triple.s.to_encoded(),
                self.counter,
                triple.g.clone().map(|g| g.as_term().clone()),
            ));
        //osp
        if !self.osp.contains_key(&triple.o.to_encoded()) {
            self.osp.insert(triple.o.to_encoded(), HashMap::new());
        }
        if !self
            .osp
            .get(&triple.o.to_encoded())
            .unwrap()
            .contains_key(&triple.s.to_encoded())
        {
            self.osp
                .get_mut(&triple.o.to_encoded())
                .unwrap()
                .insert(triple.s.to_encoded(), Vec::new());
        }
        self.osp
            .get_mut(&triple.o.to_encoded())
            .unwrap()
            .get_mut(&triple.s.to_encoded())
            .unwrap()
            .push((
                triple.p.to_encoded(),
                self.counter,
                triple.g.clone().map(|g| g.as_term().clone()),
            ));
        self.triples.push(triple);
        self.counter += 1;
    }
    pub fn contains(&self, triple: &Triple) -> bool {
        if !self.osp.contains_key(&triple.o.to_encoded()) {
            false
        } else {
            if !self
                .osp
                .get(&triple.o.to_encoded())
                .unwrap()
                .contains_key(&triple.s.to_encoded())
            {
                false
            } else {
                for (encoded, counter, _) in self
                    .osp
                    .get(&triple.o.to_encoded())
                    .unwrap()
                    .get(&triple.s.to_encoded())
                    .unwrap()
                {
                    if encoded == &triple.p.to_encoded() {
                        return true;
                    }
                }
                return false;
            }
        }
    }
    pub fn query(&self, query_triple: &Triple, triple_counter: Option<usize>) -> Option<Binding> {
        // A subject/object that's a *non-ground* list-term pattern (e.g.
        // `(:good ?Y)`) can never be found via the ordinary indexed
        // lookups below -- those compare list-term ids for exact equality,
        // but every list gets a fresh synthetic id at parse time, so a
        // pattern list is never `==` to a separately-parsed, structurally
        // compatible ground list. Route those queries through a dedicated
        // structural-unification scan instead (see `VarOrTerm::unify_list_pattern`).
        let s_is_list_pattern = query_triple.s.is_term() && VarOrTerm::is_nonground_list_pattern(query_triple.s.to_encoded());
        let o_is_list_pattern = query_triple.o.is_term() && VarOrTerm::is_nonground_list_pattern(query_triple.o.to_encoded());
        if s_is_list_pattern || o_is_list_pattern {
            return self.query_list_pattern(query_triple, triple_counter, s_is_list_pattern, o_is_list_pattern);
        }

        let mut matched_binding = Binding::new();
        let counter_check = if let Some(size) = triple_counter {
            size
        } else {
            self.counter
        };
        //?s p o
        if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.o.to_encoded()) {
                    for (encoded_match, counter, graph_name) in indexes2.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding
                                .add(&query_triple.s.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //s ?p o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()) {
                    for (encoded_match, counter, graph_name) in indexes2.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding
                                .add(&query_triple.p.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //s p ?o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.p.to_encoded()) {
                    for (encoded_match, counter, graph_name) in indexes2.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding
                                .add(&query_triple.o.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //?s ?p o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                for (s_key, p_values) in indexes.iter() {
                    for (encoded_match, counter, graph_name) in p_values.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding.add(&query_triple.s.to_encoded(), s_key.clone());
                            matched_binding
                                .add(&query_triple.p.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //s ?p ?o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()) {
                for (p_key, o_values) in indexes.iter() {
                    for (encoded_match, counter, graph_name) in o_values.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding.add(&query_triple.p.to_encoded(), p_key.clone());
                            matched_binding
                                .add(&query_triple.o.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //?s p ?o
        else if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()) {
                for (o_key, s_values) in indexes.iter() {
                    for (encoded_match, counter, graph_name) in s_values.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding.add(&query_triple.o.to_encoded(), o_key.clone());
                            matched_binding
                                .add(&query_triple.s.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //?s ?p ?o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_var() {
            for (s_key, p_index) in self.spo.iter() {
                for (p_key, o_values) in p_index.iter() {
                    for (encoded_match, counter, graph_name) in o_values.iter() {
                        if *counter <= counter_check {
                            if !Self::check_quad_match_and_add(
                                &query_triple,
                                &mut matched_binding,
                                graph_name,
                            ) {
                                break;
                            }
                            matched_binding.add(&query_triple.s.to_encoded(), s_key.clone());
                            matched_binding.add(&query_triple.p.to_encoded(), p_key.clone());
                            matched_binding
                                .add(&query_triple.o.to_encoded(), encoded_match.clone());
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        //s p o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()) {
                    for (encoded_match, counter, graph_name) in indexes2.iter() {
                        if *counter <= counter_check {
                            if *encoded_match == query_triple.p.to_encoded() {
                                // Triple matches s/p/o — check graph condition
                                if query_triple.g.is_some() {
                                    // Graph variable or term: collect/filter bindings
                                    if !Self::check_quad_match_and_add(
                                        &query_triple,
                                        &mut matched_binding,
                                        graph_name,
                                    ) {
                                        // graph term didn't match — skip but keep looking
                                        continue;
                                    }
                                    // continue to collect all matching graph bindings
                                } else {
                                    // No graph constraint — return immediately on first match
                                    return Some(matched_binding);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if matched_binding.len() > 0 {
            Some(matched_binding)
        } else {
            None
        }
    }

    /// A linear-scan fallback for queries where the subject and/or object
    /// is a non-ground list-term pattern (see `VarOrTerm::unify_list_pattern`'s
    /// doc comment for why the ordinary indexed lookups in `query` can't
    /// handle this). Correctness over speed: this crate's list-pattern
    /// corpus cases (the real EYE `good_cobbler`/`peano` cases) are small
    /// fact sets, not indexed-lookup-scale data.
    fn query_list_pattern(
        &self,
        query_triple: &Triple,
        triple_counter: Option<usize>,
        s_is_list_pattern: bool,
        o_is_list_pattern: bool,
    ) -> Option<Binding> {
        let counter_check = triple_counter.unwrap_or(self.counter);
        let mut matched_binding = Binding::new();

        for (idx, triple) in self.triples.iter().enumerate() {
            if idx > counter_check {
                break;
            }
            if query_triple.p.is_term() && triple.p.to_encoded() != query_triple.p.to_encoded() {
                continue;
            }
            if !s_is_list_pattern && query_triple.s.is_term() && triple.s.to_encoded() != query_triple.s.to_encoded() {
                continue;
            }
            if !o_is_list_pattern && query_triple.o.is_term() && triple.o.to_encoded() != query_triple.o.to_encoded() {
                continue;
            }

            let mut extra_bindings = Vec::new();
            if s_is_list_pattern
                && !VarOrTerm::unify_list_pattern(query_triple.s.to_encoded(), triple.s.to_encoded(), &mut extra_bindings)
            {
                continue;
            }
            if o_is_list_pattern
                && !VarOrTerm::unify_list_pattern(query_triple.o.to_encoded(), triple.o.to_encoded(), &mut extra_bindings)
            {
                continue;
            }

            // A variable can occur more than once inside a single list
            // pattern (e.g. the real EYE `basic-monadic` corpus case's
            // 11-element cycle-detection list `(?D0 ?D1 ... ?D9 ?D0)`,
            // where `?D0` closes the cycle by repeating as both the first
            // and last member) -- `unify_list_pattern` binds every
            // occurrence independently without checking they agree, so a
            // repeated variable must be reconciled here: reject this
            // candidate triple if any variable is bound to two different
            // values within the same match.
            {
                let mut consistent = HashMap::new();
                let mut ok = true;
                for &(var_id, val_id) in &extra_bindings {
                    match consistent.get(&var_id) {
                        Some(&existing) if existing != val_id => { ok = false; break; }
                        _ => { consistent.insert(var_id, val_id); }
                    }
                }
                if !ok {
                    continue;
                }
            }

            if query_triple.s.is_var() {
                matched_binding.add(&query_triple.s.to_encoded(), triple.s.to_encoded());
            }
            if query_triple.p.is_var() {
                matched_binding.add(&query_triple.p.to_encoded(), triple.p.to_encoded());
            }
            if query_triple.o.is_var() {
                matched_binding.add(&query_triple.o.to_encoded(), triple.o.to_encoded());
            }
            for (var_id, val_id) in extra_bindings {
                matched_binding.add(&var_id, val_id);
            }
        }

        if matched_binding.len() > 0 {
            Some(matched_binding)
        } else {
            None
        }
    }

    pub fn query_help<'a>(
        &'a self,
        query_triple: &'a Triple,
        triple_counter: Option<usize>,
    ) -> Box<dyn Iterator<Item = Vec<EncodedBinding>> + 'a> {
        //?s p o
        if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.o.to_encoded()) {
                    Self::extract_binding_values_single_var(
                        &query_triple.s,
                        &query_triple.g,
                        indexes2,
                    )
                } else {
                    Box::new(empty())
                }
            } else {
                Box::new(empty())
            }
        }
        //s ?p o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()) {
                    Self::extract_binding_values_single_var(
                        &query_triple.p,
                        &query_triple.g,
                        indexes2,
                    )
                } else {
                    Box::new(empty())
                }
            } else {
                Box::new(empty())
            }
        }
        // //s p ?o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.p.to_encoded()) {
                    Self::extract_binding_values_single_var(
                        &query_triple.o,
                        &query_triple.g,
                        indexes2,
                    )
                } else {
                    Box::new(empty())
                }
            } else {
                Box::new(empty())
            }
        }
        // //?s ?p o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                Box::new(
                    indexes
                        .iter()
                        .map(|(s_key, p_values)| {
                            p_values
                                .iter()
                                .zip(iter::repeat(s_key).take(p_values.len()))
                        })
                        .flatten()
                        .map(|((encoded_match, counter, graph_name), s_key)| {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding {
                                var: query_triple.s.to_encoded().clone(),
                                val: s_key.clone(),
                            });
                            bindings.push(EncodedBinding {
                                var: query_triple.p.to_encoded().clone(),
                                val: encoded_match.clone(),
                            });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding {
                                        var: var_name.name.clone(),
                                        val: graph_name.clone().unwrap().id(),
                                    });
                                }
                                Some(VarOrTerm::Term(term))
                                    if !graph_name.clone().map_or(false, |t| t.eq(term)) =>
                                {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        })
                        .flatten(),
                )
            } else {
                Box::new(empty())
            }
        }
        //s ?p ?o
        else if query_triple.s.is_term() & query_triple.p.is_var() & query_triple.o.is_var() {
            if let Some(indexes) = self.spo.get(&query_triple.s.to_encoded()) {
                Box::new(
                    indexes
                        .iter()
                        .map(|(key, values)| {
                            values.iter().zip(iter::repeat(key).take(values.len()))
                        })
                        .flatten()
                        .map(|((encoded_match, counter, graph_name), key)| {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding {
                                var: query_triple.p.to_encoded().clone(),
                                val: key.clone(),
                            });
                            bindings.push(EncodedBinding {
                                var: query_triple.o.to_encoded().clone(),
                                val: encoded_match.clone(),
                            });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding {
                                        var: var_name.name.clone(),
                                        val: graph_name.clone().unwrap().id(),
                                    });
                                }
                                Some(VarOrTerm::Term(term))
                                    if !graph_name.clone().map_or(false, |t| t.eq(term)) =>
                                {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        })
                        .flatten(),
                )
            } else {
                Box::new(empty())
            }
        }
        //?s p ?o
        else if query_triple.s.is_var() & query_triple.p.is_term() & query_triple.o.is_var() {
            if let Some(indexes) = self.pos.get(&query_triple.p.to_encoded()) {
                Box::new(
                    indexes
                        .iter()
                        .map(|(key, values)| {
                            values.iter().zip(iter::repeat(key).take(values.len()))
                        })
                        .flatten()
                        .map(|((encoded_match, counter, graph_name), key)| {
                            let mut bindings = Vec::with_capacity(3);
                            bindings.push(EncodedBinding {
                                var: query_triple.s.to_encoded().clone(),
                                val: encoded_match.clone(),
                            });
                            bindings.push(EncodedBinding {
                                var: query_triple.o.to_encoded().clone(),
                                val: key.clone(),
                            });

                            match &query_triple.g {
                                Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                    bindings.push(EncodedBinding {
                                        var: var_name.name.clone(),
                                        val: graph_name.clone().unwrap().id(),
                                    });
                                }
                                Some(VarOrTerm::Term(term))
                                    if !graph_name.clone().map_or(false, |t| t.eq(term)) =>
                                {
                                    return None
                                }
                                _ => {}
                            }
                            Some(bindings)
                        })
                        .flatten(),
                )
            } else {
                Box::new(empty())
            }
        }
        // //?s ?p ?o
        else if query_triple.s.is_var() & query_triple.p.is_var() & query_triple.o.is_var() {
            Box::new(
                self.spo
                    .iter()
                    .map(|(s_key, p_vals)| {
                        p_vals.iter().zip(iter::repeat(s_key).take(p_vals.len()))
                    })
                    .flatten()
                    .map(|((p_key, o_values), s_key)| {
                        o_values
                            .iter()
                            .zip(iter::repeat(p_key).take(o_values.len()))
                            .zip(iter::repeat(s_key).take(o_values.len()))
                    })
                    .flatten()
                    .map(|(((encoded_match, counter, graph_name), p_key), s_key)| {
                        let mut bindings = Vec::with_capacity(3);
                        bindings.push(EncodedBinding {
                            var: query_triple.s.to_encoded().clone(),
                            val: s_key.clone(),
                        });
                        bindings.push(EncodedBinding {
                            var: query_triple.p.to_encoded().clone(),
                            val: p_key.clone(),
                        });
                        bindings.push(EncodedBinding {
                            var: query_triple.o.to_encoded().clone(),
                            val: encoded_match.clone(),
                        });

                        match &query_triple.g {
                            Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                                bindings.push(EncodedBinding {
                                    var: var_name.name.clone(),
                                    val: graph_name.clone().unwrap().id(),
                                });
                            }
                            Some(VarOrTerm::Term(term))
                                if !graph_name.clone().map_or(false, |t| t.eq(term)) =>
                            {
                                return None
                            }
                            _ => {}
                        }
                        Some(bindings)
                    })
                    .flatten(),
            )
        }
        // //s p o
        else if query_triple.s.is_term() & query_triple.p.is_term() & query_triple.o.is_term() {
            if let Some(indexes) = self.osp.get(&query_triple.o.to_encoded()) {
                if let Some(indexes2) = indexes.get(&query_triple.s.to_encoded()) {
                    Box::new(
                        indexes2
                            .iter()
                            .map(|(encoded_match, counter, graph_name)| {
                                if *encoded_match == query_triple.p.to_encoded() {
                                    // return when triple has been found in knowlege base
                                    Some(Vec::with_capacity(0))
                                } else {
                                    None
                                }
                            })
                            .flatten(),
                    )
                } else {
                    Box::new(empty())
                }
            } else {
                Box::new(empty())
            }
        } else {
            Box::new(empty())
        }
        //
        // if matched_binding.len() > 0{
        //     Some(matched_binding)
        // }else{
        //     None
        // }
    }

    fn extract_binding_values_single_var<'a>(
        variable: &'a VarOrTerm,
        graph_var: &'a Option<VarOrTerm>,
        indexes2: &'a Vec<(usize, usize, Option<Term>)>,
    ) -> Box<dyn Iterator<Item = Vec<EncodedBinding>> + 'a> {
        Box::new(
            indexes2
                .iter()
                .map(move |(encoded_match, counter, graph_name)| {
                    let mut bindings = Vec::with_capacity(2);
                    bindings.push(EncodedBinding {
                        var: variable.to_encoded().clone(),
                        val: encoded_match.clone(),
                    });
                    match graph_var {
                        Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                            bindings.push(EncodedBinding {
                                var: var_name.name.clone(),
                                val: graph_name.clone().unwrap().id(),
                            });
                        }
                        Some(VarOrTerm::Term(term))
                            if !graph_name.clone().map_or(false, |t| t.eq(term)) =>
                        {
                            return None
                        }
                        _ => {}
                    }
                    Some(bindings)
                })
                .flatten(),
        )
    }
    fn check_quad_match_and_add(
        query_triple: &&Triple,
        matched_binding: &mut Binding,
        graph_name: &Option<Term>,
    ) -> bool {
        match &query_triple.g {
            Some(VarOrTerm::Var(var_name)) if graph_name.is_some() => {
                matched_binding.add(&var_name.name, graph_name.clone().unwrap().id());
                return true;
            }
            Some(VarOrTerm::Term(term)) if !graph_name.clone().map_or(false, |t| t.eq(term)) => {
                return false;
            }
            _ => {
                return true;
            }
        }
    }
    pub fn clear(&mut self) {
        self.triples.clear();
        self.spo.clear();
        self.osp.clear();
        self.pos.clear();
        self.counter = 0;
    }
}
#[derive(Debug, Clone)]
pub struct EncodedBinding {
    pub var: usize,
    pub val: usize,
}
pub struct QuadIterator<'a> {
    query: Triple,
    index: &'a TripleIndex,
}
impl<'a> Iterator for QuadIterator<'a> {
    type Item = Binding;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
#[cfg(test)]
#[path = "tripleindex_test.rs"]
mod tripleindex_test;
