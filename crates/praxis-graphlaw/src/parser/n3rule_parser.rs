use crate::registry::SYNTHETIC_COUNTER;
use crate::{BodyLiteral, Rule as ReasonerRule, Triple, VarOrTerm};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

use pest::iterators::{Pair, Pairs};
use pest::Parser;

// ---------------------------------------------------------------------------
// `@forAll`/`@forSome` quantifier scoping
// ---------------------------------------------------------------------------
//
// Real per-formula scoping per the N3 CG spec: a variable named in an
// `@forAll` declared *within* some formula (the document root, or a nested
// `{ ... }`) is universally quantified and scoped to that formula -- reusing
// the same bare name in an unrelated sibling formula that *also* explicitly
// quantifies it must produce a genuinely distinct variable. A variable named
// in `@forSome` is existentially quantified and is skolemized (replaced with
// a fresh blank node) at parse time, scoped the same way.
//
// Deliberate scope of this implementation (documented conflict discovered
// while implementing): this only affects variables an author *explicitly*
// names in a `@forAll`/`@forSome` declaration. Bare/unquantified variables
// keep this engine's pre-existing flat, name-based identity across formula
// boundaries. That flat behaviour is not just an unrelated legacy quirk --
// `reasoner/log_implies.rs`'s dynamic `log:implies` reification is
// *structurally* built on it: its own doc comment states variables are
// "matched across antecedent/consequent/outer scopes purely by *name*", and
// `n3_scoping.rs`'s existing `test_chained_implication_through_log_implies_*`
// tests lock in exactly that -- a bare `?citizen` used in one top-level
// quoted formula (`:alice :says { ?citizen a :GoodCitizen }`) must resolve
// to the SAME variable as the bare `?citizen` used in a completely different
// formula nested inside a rule body (`?formula log:implies { ?citizen a
// :TaxPayer }`) and the rule's own head, or the derivation silently fails to
// ground. Auto-scoping every bare variable to its own formula (the naive
// reading of "sibling formulas must not collide") breaks that mechanism.
// Scoping only names an author has *explicitly* opted into quantifying has
// no such conflict, since `log:implies` fixtures never declare `@forAll`/
// `@forSome`, and it is the literal, spec-accurate meaning of those two
// declarations -- so that is what is implemented here.
#[derive(Default)]
struct FormulaScope {
    /// `@forAll`-declared name -> fresh variable name scoped to this formula.
    forall: HashMap<String, String>,
    /// `@forSome`-declared name -> skolemized blank-node term scoped to this formula.
    forsome: HashMap<String, VarOrTerm>,
}

#[derive(Default)]
struct ScopeStack {
    scopes: Vec<FormulaScope>,
    counter: usize,
}

impl ScopeStack {
    fn push(&mut self) {
        self.scopes.push(FormulaScope::default());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    /// Register `name` as `@forAll`-quantified in the current (innermost)
    /// scope, generating its fresh formula-scoped variable name.
    fn declare_forall(&mut self, name: &str) {
        self.counter += 1;
        let renamed = format!("{}__forall{}", name, self.counter);
        if let Some(scope) = self.scopes.last_mut() {
            scope.forall.insert(name.to_string(), renamed);
        }
    }

    /// Register `name` as `@forSome`-quantified in the current (innermost)
    /// scope, skolemizing it to a fresh blank node immediately.
    fn declare_forsome(&mut self, name: &str) {
        self.counter += 1;
        let tag = SYNTHETIC_COUNTER.fetch_add(1, Ordering::SeqCst);
        let skolem = VarOrTerm::new_blank_node(format!("__n3forsome_{}_{}", tag, self.counter));
        if let Some(scope) = self.scopes.last_mut() {
            scope.forsome.insert(name.to_string(), skolem);
        }
    }

    /// Resolve a bare variable name (without the leading `?`) to its
    /// `VarOrTerm`, honoring any explicit `@forAll`/`@forSome` declaration
    /// found in the current formula-scope stack (innermost scope wins).
    /// Names never explicitly quantified fall through unchanged (this
    /// engine's pre-existing flat, name-based identity -- see the module
    /// doc comment above for why).
    fn resolve(&self, name: &str) -> VarOrTerm {
        for scope in self.scopes.iter().rev() {
            if let Some(term) = scope.forsome.get(name) {
                return term.clone();
            }
        }
        for scope in self.scopes.iter().rev() {
            if let Some(renamed) = scope.forall.get(name) {
                return VarOrTerm::new_var(renamed.clone());
            }
        }
        VarOrTerm::new_var(name.to_string())
    }
}

thread_local! {
    static SCOPE_STACK: RefCell<ScopeStack> = RefCell::new(ScopeStack::default());
}

/// Push a fresh formula scope for the duration of `f` (used both for the
/// document root and for each nested `Formula`-as-term), popping it again
/// afterwards even if `f` panics.
fn with_new_scope<T>(f: impl FnOnce() -> T) -> T {
    SCOPE_STACK.with(|s| s.borrow_mut().push());
    let result = f();
    SCOPE_STACK.with(|s| s.borrow_mut().pop());
    result
}

fn declare_forall_var(name: &str) {
    SCOPE_STACK.with(|s| s.borrow_mut().declare_forall(name));
}

fn declare_forsome_var(name: &str) {
    SCOPE_STACK.with(|s| s.borrow_mut().declare_forsome(name));
}

/// Resolve a bare variable name (leading `?` already stripped) through the
/// current formula-scope stack. The single call site is `make_term`, which
/// is itself the sole funnel for every `Var` pair in the grammar (subject,
/// object, property, list member, formula member, ...), so this covers all
/// variable occurrences uniformly.
fn resolve_var(name: &str) -> VarOrTerm {
    SCOPE_STACK.with(|s| s.borrow().resolve(name))
}

/// Scan a `ForAll`/`ForSome` pair's inner `Var` children and register each
/// declared name in the CURRENT (innermost) scope. Called in a pre-pass over
/// a formula's (or the document's) direct children before processing its
/// ordinary triples/rules, so declarations take effect regardless of where
/// textually within the formula they appear.
fn register_quantifier_declarations(pair: &Pair<Rule>) {
    let is_forall = pair.as_rule() == Rule::ForAll;
    for child in pair.clone().into_inner() {
        if child.as_rule() == Rule::Var {
            let name = &child.as_str()[1..]; // strip leading '?'
            if is_forall {
                declare_forall_var(name);
            } else {
                declare_forsome_var(name);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pest-generated parser
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "parser/n3.pest"]
pub struct N3Parser;

// ---------------------------------------------------------------------------
// Prefix mapper
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PrefixMapper {
    prefixes: HashMap<String, String>,
    /// The current `@base`/BASE IRI, if any has been declared so far in the
    /// document. `None` means no base is in effect, in which case `resolve`
    /// is a no-op -- matching this engine's pre-existing behaviour of taking
    /// bracketed IRIs verbatim when no base has been declared.
    base: Option<String>,
}

impl PrefixMapper {
    pub fn new() -> PrefixMapper {
        PrefixMapper {
            prefixes: HashMap::new(),
            base: None,
        }
    }

    pub fn add(&mut self, prefix: String, full_iri: String) {
        // A `@prefix`/PREFIX IRI can itself be a relative reference, resolved
        // against the current base at the point of declaration (RFC 3986 +
        // N3/SPARQL semantics).
        let resolved = self.resolve(&full_iri);
        self.prefixes.insert(prefix, resolved);
    }

    /// Set (or update) the current base IRI, resolving it against whatever
    /// base was already in effect if it is itself a relative reference.
    pub fn set_base(&mut self, iri: &str) {
        let resolved = self.resolve(iri);
        self.base = Some(resolved);
    }

    /// Resolve a raw IRI reference (as it appeared inside `<...>`, unbracketed)
    /// against the current base, per RFC 3986 sec. 5. If no base is in effect,
    /// the reference is returned unchanged (an absolute reference is also
    /// returned unchanged, since `resolve_reference` recognizes its scheme).
    pub fn resolve(&self, iri: &str) -> String {
        match &self.base {
            Some(base) => resolve_reference(base, iri),
            None => iri.to_string(),
        }
    }

    /// Expand a prefixed name, a bare `a`, or a `<IRI>` reference.
    /// Returns the canonical `<IRI>` form.
    pub fn expand(&self, raw: &str) -> String {
        // Trim whitespace and trailing dots (N3 TP terminator may have been consumed)
        let trimmed = raw.trim().trim_end_matches('.');

        // rdf:type shorthand
        if trimmed == "a" {
            return "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string();
        }

        // Already a full <IRI>
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            return trimmed.to_string();
        }

        // Prefixed name (prefix:local) — local may also have trailing dots that were consumed
        if let Some(colon) = trimmed.find(':') {
            let prefix = &trimmed[..colon];
            // Strip any residual trailing dot from the local name
            let local = trimmed[colon + 1..].trim_end_matches('.');
            if let Some(expanded) = self.prefixes.get(prefix) {
                return format!("<{}{}>", expanded, local);
            }
        }

        // Return as-is (e.g., variable or unknown term)
        trimmed.to_string()
    }
}

impl Default for PrefixMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RFC 3986 sec. 5 IRI-reference resolution
// ---------------------------------------------------------------------------

/// Split `s` into (scheme, rest-after-colon) if `s` starts with a valid
/// `scheme:` prefix (RFC 3986 sec. 3.1), else `None`.
fn split_scheme(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return None;
    }
    for (i, b) in bytes.iter().enumerate() {
        match b {
            b':' => {
                if i == 0 {
                    return None;
                }
                return Some((&s[..i], &s[i + 1..]));
            }
            b if b.is_ascii_alphanumeric() || *b == b'+' || *b == b'-' || *b == b'.' => continue,
            _ => return None,
        }
    }
    None
}

/// Split off a `#fragment` suffix, if present.
fn split_fragment(s: &str) -> (&str, Option<&str>) {
    match s.find('#') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    }
}

/// Split off a `?query` suffix (input must already have any fragment removed).
fn split_query(s: &str) -> (&str, Option<&str>) {
    match s.find('?') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    }
}

/// Split `//authority/path...` into (Some(authority), "/path...") -- or, if
/// `s` doesn't start with `//`, (None, s) unchanged.
fn split_authority(s: &str) -> (Option<&str>, &str) {
    if let Some(rest) = s.strip_prefix("//") {
        let end = rest.find('/').unwrap_or(rest.len());
        (Some(&rest[..end]), &rest[end..])
    } else {
        (None, s)
    }
}

/// RFC 3986 sec. 5.2.4 `remove_dot_segments`.
fn remove_dot_segments(path: &str) -> String {
    let mut input = path;
    let mut output = String::new();
    while !input.is_empty() {
        if let Some(rest) = input.strip_prefix("../") {
            input = rest;
        } else if let Some(rest) = input.strip_prefix("./") {
            input = rest;
        } else if let Some(rest) = input.strip_prefix("/./") {
            input = rest;
            output.push('/');
        } else if input == "/." {
            input = "/";
        } else if let Some(rest) = input.strip_prefix("/../") {
            input = rest;
            if let Some(pos) = output.rfind('/') {
                output.truncate(pos);
            } else {
                output.clear();
            }
            output.push('/');
        } else if input == "/.." {
            input = "/";
            if let Some(pos) = output.rfind('/') {
                output.truncate(pos);
            } else {
                output.clear();
            }
        } else if input == "." || input == ".." {
            input = "";
        } else {
            // Move the first path segment to the output, including its
            // leading "/" (if any) but stopping *before* (not consuming) the
            // next "/" -- that terminating slash must stay at the front of
            // `input` so the next iteration can recognize a following
            // "/../" or "/./" prefix (consuming it here would misparse e.g.
            // "/a/b/c/../g": the correctly-RFC-specified rewrite to "/a/b/g"
            // depends on each "/../" being seen with its leading "/" intact).
            let start = input.strip_prefix('/').unwrap_or(input);
            let seg_end = start.find('/').unwrap_or(start.len());
            let prefix_len = input.len() - start.len();
            let take = prefix_len + seg_end;
            output.push_str(&input[..take]);
            input = &input[take..];
        }
    }
    output
}

/// Merge a relative-reference path with the base path per RFC 3986 sec. 5.3.
fn merge_path(base_has_authority: bool, base_path: &str, ref_path: &str) -> String {
    if base_has_authority && base_path.is_empty() {
        format!("/{}", ref_path)
    } else if let Some(pos) = base_path.rfind('/') {
        format!("{}{}", &base_path[..=pos], ref_path)
    } else {
        ref_path.to_string()
    }
}

fn build_iri(
    scheme: &str,
    authority: Option<&str>,
    path: &str,
    query: Option<&str>,
    fragment: Option<&str>,
) -> String {
    let mut out = String::new();
    out.push_str(scheme);
    out.push(':');
    if let Some(auth) = authority {
        out.push_str("//");
        out.push_str(auth);
    }
    out.push_str(path);
    if let Some(q) = query {
        out.push('?');
        out.push_str(q);
    }
    if let Some(f) = fragment {
        out.push('#');
        out.push_str(f);
    }
    out
}

/// Resolve `reference` (a possibly-relative IRI reference) against `base` (an
/// absolute IRI), implementing the RFC 3986 sec. 5.3 "Transform References"
/// algorithm (component-wise; the "strict" variant -- this engine has no need
/// for the backward-compatible non-strict `.` scheme quirk).
fn resolve_reference(base: &str, reference: &str) -> String {
    if let Some((r_scheme, r_after_scheme)) = split_scheme(reference) {
        let (r_no_frag, r_frag) = split_fragment(r_after_scheme);
        let (r_no_query, r_query) = split_query(r_no_frag);
        let (r_auth, r_path) = split_authority(r_no_query);
        let path = remove_dot_segments(r_path);
        return build_iri(r_scheme, r_auth, &path, r_query, r_frag);
    }

    let Some((b_scheme, b_after_scheme)) = split_scheme(base) else {
        // Base itself has no scheme (shouldn't normally happen) -- fall back
        // to returning the reference unresolved rather than panicking.
        return reference.to_string();
    };
    let (b_no_frag, _b_frag) = split_fragment(b_after_scheme);
    let (b_no_query, _b_query) = split_query(b_no_frag);
    let (b_auth, b_path) = split_authority(b_no_query);

    let (ref_no_frag, r_frag) = split_fragment(reference);

    if ref_no_frag.is_empty() {
        // Same-document reference: just the fragment changes.
        return build_iri(b_scheme, b_auth, b_path, None, r_frag);
    }

    if let Some(rest) = ref_no_frag.strip_prefix("//") {
        // Network-path reference: keep scheme, take reference's authority/path/query.
        let end = rest.find('/').unwrap_or(rest.len());
        let r_auth = &rest[..end];
        let (r_path_query, r_query) = split_query(&rest[end..]);
        let path = remove_dot_segments(r_path_query);
        return build_iri(b_scheme, Some(r_auth), &path, r_query, r_frag);
    }

    let (r_path_query, r_query) = split_query(ref_no_frag);

    if let Some(stripped) = r_path_query.strip_prefix('/') {
        let path = remove_dot_segments(&format!("/{}", stripped));
        return build_iri(b_scheme, b_auth, &path, r_query, r_frag);
    }

    if r_path_query.is_empty() {
        return build_iri(b_scheme, b_auth, b_path, r_query, r_frag);
    }

    let merged = merge_path(b_auth.is_some(), b_path, r_path_query);
    let path = remove_dot_segments(&merged);
    build_iri(b_scheme, b_auth, &path, r_query, r_frag)
}

// ---------------------------------------------------------------------------
// Term building helpers
// ---------------------------------------------------------------------------

/// Convert a raw string value (already expanded) into a VarOrTerm.
///
/// NOTE: the fallback branch deliberately calls `VarOrTerm::new_term` directly
/// rather than `VarOrTerm::convert`. `convert` wraps any string that isn't
/// already `<...>`/`"..."`/`_:...` in angle brackets -- which is exactly right
/// for a *bare* prefixed name with no matching `@prefix` (e.g. "test:Foo" with
/// no `@prefix test:` declared, where `PrefixMapper::expand` returns the text
/// unchanged as a fallback). Using `new_term` here keeps that fallback case
/// encoded identically to the legacy line-based parser (`Parser::parse`),
/// which also interns such tokens raw/unwrapped. This matters because
/// `TripleStore::from` tries this pest-based parser first and falls back to
/// the legacy parser on failure -- if both parsers can succeed on the same
/// kind of document, they must agree on term encoding or a test comparing
/// pre-existing (legacy-encoded) terms against freshly-parsed (pest-encoded)
/// ones would silently break.
fn make_term(raw: &str) -> VarOrTerm {
    let trimmed = raw.trim();
    if trimmed.starts_with('?') {
        // Strip the leading '?' to match VarOrTerm::convert("?x") behaviour which stores "x"
        resolve_var(&trimmed[1..])
    } else if trimmed.starts_with("_:") {
        VarOrTerm::new_blank_node(trimmed[2..].to_string())
    } else {
        VarOrTerm::new_term(trimmed.to_string())
    }
}

/// Parse a literal pest pair into a VarOrTerm literal.
///
/// NOTE: numeric/boolean literals are encoded via `VarOrTerm::new_literal` with
/// a proper xsd datatype (mirroring the string-literal handling just below),
/// **not** via `VarOrTerm::convert`. `convert` would wrap a bare lexical form
/// like "42" in angle brackets ("<42>"), i.e. treat it as an opaque IRI-like
/// token instead of a numeric value -- which would make it impossible for the
/// math:* built-ins (queryengine.rs) to recover a numeric value from it.
fn parse_literal_pair(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    // The outer Literal rule may contain StringValue + optional LangTag / DatatypeAnnotation,
    // or a numeric / boolean literal.
    let raw = pair.as_str().trim().to_string();
    let mut inner = pair.into_inner().peekable();

    if let Some(first) = inner.peek() {
        match first.as_rule() {
            Rule::StringValue => {
                let string_pair = inner.next().unwrap();
                let lex = unescape_string(string_pair.as_str());

                // Check for lang tag or datatype
                if let Some(annotation) = inner.next() {
                    match annotation.as_rule() {
                        Rule::LangTag => {
                            // @en → strip the @
                            let lang = &annotation.as_str()[1..];
                            return VarOrTerm::new_literal(lex, None, Some(lang.to_string()));
                        }
                        Rule::DatatypeAnnotation => {
                            // "^^" has been consumed by pest; the child is either a
                            // bare "<IRI>" (Rule::Iri) or a "prefix:local" name
                            // (Rule::Prefixed) that must be expanded via `prefixes`.
                            let dt_str = match annotation.into_inner().next() {
                                Some(p) if p.as_rule() == Rule::BracketedIri => {
                                    let iri = p
                                        .into_inner()
                                        .next()
                                        .map(|q| q.as_str())
                                        .unwrap_or("");
                                    format!("<{}>", prefixes.resolve(iri))
                                }
                                Some(p) if p.as_rule() == Rule::Prefixed => prefixes.expand(p.as_str()),
                                _ => String::new(),
                            };
                            return VarOrTerm::new_literal(lex, Some(dt_str), None);
                        }
                        _ => {}
                    }
                }
                // Plain string literal → xsd:string
                let xsd_string = "<http://www.w3.org/2001/XMLSchema#string>".to_string();
                return VarOrTerm::new_literal(lex, Some(xsd_string), None);
            }
            Rule::IntegerLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#integer>".to_string()),
                    None,
                );
            }
            Rule::DecimalLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#decimal>".to_string()),
                    None,
                );
            }
            Rule::DoubleLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#double>".to_string()),
                    None,
                );
            }
            Rule::BoolLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#boolean>".to_string()),
                    None,
                );
            }
            _ => {}
        }
    }

    // Should not normally be reached given the Literal grammar's alternatives.
    VarOrTerm::convert(raw)
}

/// Parse an RDF list ("(" ListItem* ")") into a single VarOrTerm list term.
/// Any bnode-property-list members (`[ ... ]`) nested inside the list
/// contribute their desugared property triples into `extra`, exactly as
/// they would in Subject/Object position.
fn parse_list(pair: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    let mut members = Vec::new();
    for list_item in pair.into_inner() {
        if let Some(child) = list_item.into_inner().next() {
            members.push(term_from_pair(child, prefixes, extra));
        }
    }
    VarOrTerm::new_list(members)
}

/// Mint a fresh synthetic blank node, reusing the process-wide
/// `SYNTHETIC_COUNTER` (same mechanism `VarOrTerm::new_list`/`new_formula`
/// use) so the label can never collide with a user-written `_:label` or
/// another synthetic term.
fn fresh_bnode() -> VarOrTerm {
    let tag = SYNTHETIC_COUNTER.fetch_add(1, Ordering::SeqCst);
    VarOrTerm::new_blank_node(format!("__n3bnodeprops_{}", tag))
}

/// Parse an anonymous blank-node property list ("[" PredicateObjectList? "]")
/// into a fresh synthetic blank node, pushing its desugared property triples
/// into `extra`. An empty "[]" (no PredicateObjectList child) yields just the
/// fresh blank node with no additional triples -- matching Turtle/N3
/// semantics exactly.
fn parse_bnode_props(pair: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    let bnode = fresh_bnode();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::PredicateObjectList {
            parse_predicate_object_list(bnode.clone(), child, prefixes, extra);
        }
    }
    bnode
}

/// Shared logic for a "property object[, object]* [; property object[, object]*]*"
/// production attached to `subject_vot` (used both for a TP's top-level
/// PredicateObjectList and for a `[ ... ]` bnode property list's inner one).
/// Pushes one triple per (property, object) pair into `extra`.
fn parse_predicate_object_list(
    subject_vot: VarOrTerm,
    pair: Pair<Rule>,
    prefixes: &PrefixMapper,
    extra: &mut Vec<Triple>,
) {
    let mut property_vot = VarOrTerm::new_var("p".to_string());
    let mut objects_vot: Vec<VarOrTerm> = Vec::new();
    let flush = |property_vot: &VarOrTerm, objects_vot: &mut Vec<VarOrTerm>, triples: &mut Vec<Triple>| {
        if objects_vot.is_empty() {
            objects_vot.push(VarOrTerm::new_var("o".to_string()));
        }
        for o in objects_vot.drain(..) {
            triples.push(Triple { s: subject_vot.clone(), p: property_vot.clone(), o, g: None });
        }
    };
    for pol_part in pair.into_inner() {
        match pol_part.as_rule() {
            Rule::Property => {
                if !objects_vot.is_empty() {
                    flush(&property_vot, &mut objects_vot, extra);
                }
                let expanded = expand_property(pol_part, prefixes);
                property_vot = make_term(&expanded);
            }
            Rule::ObjectList => {
                for obj_pair in pol_part.into_inner() {
                    if obj_pair.as_rule() == Rule::Object {
                        objects_vot.push(parse_object(obj_pair, prefixes, extra));
                    }
                }
            }
            _ => {}
        }
    }
    flush(&property_vot, &mut objects_vot, extra);
}

/// Parse a quoted graph ("{" (ForAll | ForSome | TP)* "}") into a single
/// VarOrTerm formula term. Pushes a fresh formula-scope so any `@forAll`/
/// `@forSome` declared directly inside this formula scopes those variables
/// to it (see the `ScopeStack` module doc comment) rather than leaking into
/// an enclosing or sibling formula.
fn parse_formula(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    with_new_scope(|| {
        let children: Vec<Pair<Rule>> = pair.into_inner().collect();
        // Pre-pass: register this formula's own quantifier declarations
        // before parsing its triples, so order-within-the-formula doesn't
        // matter (a declaration after its first use still applies).
        for child in &children {
            if child.as_rule() == Rule::ForAll || child.as_rule() == Rule::ForSome {
                register_quantifier_declarations(child);
            }
        }
        let mut triples = Vec::new();
        for tp_pair in children {
            if tp_pair.as_rule() == Rule::TP {
                triples.extend(parse_tp(tp_pair.into_inner(), prefixes));
            }
        }
        VarOrTerm::new_formula(triples)
    })
}

/// Extract the raw (unbracketed) IRI-reference text from an `IriRef`
/// (`"<" ~ IriReference ~ ">"`, wrapped in the atomic `BracketedIri`) pair.
fn bracketed_iri_text(iri_ref_pair: Pair<Rule>) -> &str {
    iri_ref_pair
        .into_inner()
        .next()
        .and_then(|bracketed| bracketed.into_inner().next())
        .map(|iri_reference| iri_reference.as_str())
        .unwrap_or("")
}

/// Shared term-building logic for anything that can appear in a Subject or
/// Object position (IRI, prefixed name, variable, blank node, literal, list,
/// or quoted graph).
fn term_from_pair(child: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    match child.as_rule() {
        Rule::IriRef => {
            let iri = bracketed_iri_text(child);
            make_term(&format!("<{}>", prefixes.resolve(iri)))
        }
        Rule::Prefixed => make_term(&prefixes.expand(child.as_str())),
        Rule::Var => make_term(child.as_str()),
        Rule::BlankNode => make_term(child.as_str()),
        Rule::Literal => parse_literal_pair(child, prefixes),
        Rule::List => parse_list(child, prefixes, extra),
        Rule::Formula => parse_formula(child, prefixes),
        Rule::BNodeProps => parse_bnode_props(child, prefixes, extra),
        Rule::PathExpr => parse_path_expr(child, prefixes, extra),
        _ => make_term(child.as_str()),
    }
}

/// Mint a fresh synthetic blank node for a path-syntax existential (`x!p` /
/// `x^p`), reusing the process-wide `SYNTHETIC_COUNTER` so its label can
/// never collide with a user-written `_:label` or any other synthetic term.
fn fresh_path_bnode() -> VarOrTerm {
    let tag = SYNTHETIC_COUNTER.fetch_add(1, Ordering::SeqCst);
    VarOrTerm::new_blank_node(format!("__n3path_{}", tag))
}

/// Parse a `PathPredicate` (`IriRef | Prefixed | Var`) into the term used as
/// a path segment's predicate.
fn parse_path_predicate(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    match pair.into_inner().next() {
        Some(child) => match child.as_rule() {
            Rule::IriRef => {
                let iri = bracketed_iri_text(child);
                make_term(&format!("<{}>", prefixes.resolve(iri)))
            }
            Rule::Prefixed => make_term(&prefixes.expand(child.as_str())),
            Rule::Var => make_term(child.as_str()),
            _ => make_term(child.as_str()),
        },
        None => make_term(""),
    }
}

/// Parse a `PathExpr` (`PathHead ~ PathSegment+`) into the term the whole
/// path expression evaluates to, pushing one desugared triple per segment
/// into `extra`. `x!p` desugars to a fresh existential `_:v` plus the triple
/// `(x, p, _:v)`; `x^p` (inverse path) desugars to a fresh existential `_:v`
/// plus the triple `(_:v, p, x)`. Chained segments (mixing `!`/`^` freely,
/// e.g. `x!p^q`) fold left-to-right: each segment's fresh existential
/// becomes the base term for the next segment.
fn parse_path_expr(pair: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    let mut inner = pair.into_inner();
    let head_pair = inner.next().expect("PathExpr must have a PathHead");
    let head_child = head_pair
        .into_inner()
        .next()
        .expect("PathHead must have a child term");
    let mut current = term_from_pair(head_child, prefixes, extra);
    for segment in inner {
        let mut seg_inner = segment.into_inner();
        let dir_pair = seg_inner.next().expect("PathSegment must have a direction");
        let pred_pair = seg_inner
            .next()
            .expect("PathSegment must have a PathPredicate");
        let predicate = parse_path_predicate(pred_pair, prefixes);
        let fresh = fresh_path_bnode();
        match dir_pair.as_rule() {
            Rule::PathInverse => extra.push(Triple {
                s: fresh.clone(),
                p: predicate,
                o: current,
                g: None,
            }),
            // PathForward, and any other case (shouldn't occur per grammar).
            _ => extra.push(Triple {
                s: current,
                p: predicate,
                o: fresh.clone(),
                g: None,
            }),
        }
        current = fresh;
    }
    current
}

/// Strip surrounding quotes from a string literal and decode N3/Turtle
/// string escape sequences (`\n \r \t \b \f \" \' \\ \uXXXX \UXXXXXXXX`).
fn unescape_string(raw: &str) -> String {
    let s = raw.trim();
    let inner = if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") && s.len() >= 6 {
        &s[3..s.len() - 3]
    } else if s.starts_with("'''") && s.ends_with("'''") && s.len() >= 6 {
        &s[3..s.len() - 3]
    } else if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    };
    decode_escapes(inner)
}

/// Decode N3/Turtle string escape sequences in an already-unquoted string.
///
/// Supported escapes: `\t \n \r \b \f \" \' \\ \uXXXX \UXXXXXXXX`. Any other
/// `\x` sequence is left as-is (backslash + char preserved) since it is not a
/// recognized escape.
fn decode_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('u') => {
                if let Some(ch) = take_hex_escape(&mut chars, 4) {
                    out.push(ch);
                } else {
                    out.push('\\');
                    out.push('u');
                }
            }
            Some('U') => {
                if let Some(ch) = take_hex_escape(&mut chars, 8) {
                    out.push(ch);
                } else {
                    out.push('\\');
                    out.push('U');
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Consume exactly `n` hex digits from `chars` and decode them as a Unicode
/// code point. Returns `None` (consuming nothing further) if fewer than `n`
/// hex digits are available or the resulting code point is invalid, in which
/// case the caller falls back to emitting the escape literally.
fn take_hex_escape(chars: &mut std::iter::Peekable<std::str::Chars>, n: usize) -> Option<char> {
    let mut digits = String::with_capacity(n);
    let mut lookahead = chars.clone();
    for _ in 0..n {
        match lookahead.next() {
            Some(c) if c.is_ascii_hexdigit() => digits.push(c),
            _ => return None,
        }
    }
    let code = u32::from_str_radix(&digits, 16).ok()?;
    let ch = char::from_u32(code)?;
    // Only consume from the real iterator once we know decoding succeeded.
    for _ in 0..n {
        chars.next();
    }
    Some(ch)
}

// ---------------------------------------------------------------------------
// Triple pattern parsing
// ---------------------------------------------------------------------------

/// Parse a single TP (triple pattern) production into one or more `Triple`s.
/// More than one triple results from comma sugar in an object list ("s p o1,
/// o2, o3 .") and/or semicolon sugar across predicate-object pairs ("s p1
/// o1; p2 o2 .") -- every resulting triple shares the same subject; a
/// semicolon segment's triples additionally share that segment's property.
/// `InverseTP` ("object is predicate of subject .") and `HasTP` ("subject
/// has predicate object .") are both real N3/EYE sugar for the ordinary
/// triple (subject, predicate, object).
fn parse_tp(pairs: Pairs<'_, Rule>, prefixes: &PrefixMapper) -> Vec<Triple> {
    let mut subject_vot = VarOrTerm::new_var("s".to_string());
    // Shared sink for both this TP's own (property, object) triples and any
    // extra triples desugared from `[ ... ]` bnode property lists nested in
    // its subject/object positions (or inside a List member).
    let mut triples = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::Subject => {
                subject_vot = parse_subject(pair, prefixes, &mut triples);
            }
            Rule::PredicateObjectList => {
                parse_predicate_object_list(subject_vot.clone(), pair, prefixes, &mut triples);
            }
            // Inverse-predicate sugar: "object is predicate of subject ."
            // desugars to the ordinary triple (subject, predicate, object).
            Rule::InverseTP => {
                let mut inner = pair.into_inner();
                let object_pair = inner.next().expect("InverseTP must have an Object");
                let property_pair = inner.next().expect("InverseTP must have a Property");
                let subject_pair = inner.next().expect("InverseTP must have a Subject");
                let object_vot = parse_object(object_pair, prefixes, &mut triples);
                let property_vot = make_term(&expand_property(property_pair, prefixes));
                let subject_vot2 = parse_subject(subject_pair, prefixes, &mut triples);
                triples.push(Triple { s: subject_vot2, p: property_vot, o: object_vot, g: None });
            }
            // `has` sugar: "subject has predicate object ." desugars to the
            // same ordinary triple (subject, predicate, object) -- `has` is
            // purely a readability filler word.
            Rule::HasTP => {
                let mut inner = pair.into_inner();
                let subject_pair = inner.next().expect("HasTP must have a Subject");
                let property_pair = inner.next().expect("HasTP must have a Property");
                let object_pair = inner.next().expect("HasTP must have an Object");
                let subject_vot2 = parse_subject(subject_pair, prefixes, &mut triples);
                let property_vot = make_term(&expand_property(property_pair, prefixes));
                let object_vot = parse_object(object_pair, prefixes, &mut triples);
                triples.push(Triple { s: subject_vot2, p: property_vot, o: object_vot, g: None });
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    triples
}

fn parse_subject(pair: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    match pair.into_inner().next() {
        Some(child) => term_from_pair(child, prefixes, extra),
        None => VarOrTerm::new_var("s".to_string()),
    }
}

fn expand_property(pair: Pair<Rule>, prefixes: &PrefixMapper) -> String {
    let inner = pair.into_inner().next();
    if let Some(child) = inner {
        match child.as_rule() {
            Rule::RdfType => "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),
            // "=" sugar for owl:sameAs in Property position (real N3 idiom).
            Rule::SameAs => "<http://www.w3.org/2002/07/owl#sameAs>".to_string(),
            Rule::IriRef => {
                let iri = bracketed_iri_text(child);
                format!("<{}>", prefixes.resolve(iri))
            }
            Rule::Prefixed => prefixes.expand(child.as_str()),
            Rule::Var => child.as_str().to_string(),
            _ => child.as_str().to_string(),
        }
    } else {
        String::new()
    }
}

fn parse_object(pair: Pair<Rule>, prefixes: &PrefixMapper, extra: &mut Vec<Triple>) -> VarOrTerm {
    match pair.into_inner().next() {
        Some(child) => term_from_pair(child, prefixes, extra),
        None => VarOrTerm::new_var("o".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Public parse function
// ---------------------------------------------------------------------------

/// Parse a complete N3 document into its plain (non-rule) fact triples and
/// its rules, in one unified pest-based pass.
///
/// Supports:
/// - `@prefix` declarations (anywhere in the document)
/// - Plain top-level fact triples ("s p o .", including comma-sugar object lists)
/// - `{body} => {head} .` rules with negated literals (`not { TP }`)
/// - Named IRIs (`<...>`), prefixed names, variables (`?name`), blank nodes (`_:x`)
/// - String, numeric, boolean, lang-tagged, and datatyped literals
/// - RDF lists (`( a b c )`) and quoted graphs (`{ a b c }`) used as terms
/// - Multi-triple heads
/// - `#` line comments
///
/// Returns `Err(String)` on parse failure.
pub fn parse_document(input: &str) -> Result<(Vec<Triple>, Vec<ReasonerRule>), String> {
    let parsed = N3Parser::parse(Rule::document, input).map_err(|e| format!("N3 parse error: {}", e))?;

    let document = match parsed.into_iter().next() {
        Some(p) => p,
        // The document root is itself a formula scope: any `@forAll`/
        // `@forSome` declared at the top level (outside any nested `{ }`)
        // scopes those variables document-wide (see the `ScopeStack` module
        // doc comment above `make_term`/`resolve_var`).
        None => return with_new_scope(|| Ok((Vec::new(), Vec::new()))),
    };

    with_new_scope(|| parse_document_body(document))
}

fn parse_document_body(document: Pair<Rule>) -> Result<(Vec<Triple>, Vec<ReasonerRule>), String> {
    let mut rules: Vec<ReasonerRule> = Vec::new();
    let mut content: Vec<Triple> = Vec::new();
    let mut prefix_mapper = PrefixMapper::new();

    let items: Vec<Pair<Rule>> = document.into_inner().collect();

    // Pre-pass: register every document-root `@forAll`/`@forSome` before
    // processing anything else, so declarations take effect regardless of
    // where textually in the document they appear (matching the same
    // pre-pass done per-`Formula` in `parse_formula`).
    for item in &items {
        if item.as_rule() == Rule::ForAll || item.as_rule() == Rule::ForSome {
            register_quantifier_declarations(item);
        }
    }

    for item in items {
        match item.as_rule() {
            // Declarations were already registered in the pre-pass above.
            Rule::ForAll | Rule::ForSome => {}
            Rule::Prefix => {
                let mut prefix_name = String::new();
                let mut prefix_iri = String::new();
                for child in item.into_inner() {
                    match child.as_rule() {
                        Rule::PrefixIdentifier => prefix_name = child.as_str().to_string(),
                        Rule::BracketedIri => {
                            prefix_iri = child
                                .into_inner()
                                .next()
                                .map(|p| p.as_str().to_string())
                                .unwrap_or_default();
                        }
                        _ => {}
                    }
                }
                prefix_mapper.add(prefix_name, prefix_iri);
            }

            // `@base <IRI> .` -- sets/updates the current base IRI (which may
            // itself be a relative reference, resolved against whatever base
            // was already in effect: see `PrefixMapper::set_base`).
            Rule::Base => {
                if let Some(iri_pair) = item
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::BracketedIri)
                    .and_then(|bracketed| bracketed.into_inner().next())
                {
                    prefix_mapper.set_base(iri_pair.as_str());
                }
            }

            // SPARQL-style `PREFIX p: <IRI>` -- same semantics as `@prefix`,
            // just without the leading `@`/trailing `.`.
            Rule::SparqlPrefix => {
                let mut prefix_name = String::new();
                let mut prefix_iri = String::new();
                for child in item.into_inner() {
                    match child.as_rule() {
                        Rule::PrefixIdentifier => prefix_name = child.as_str().to_string(),
                        Rule::BracketedIri => {
                            prefix_iri = child
                                .into_inner()
                                .next()
                                .map(|p| p.as_str().to_string())
                                .unwrap_or_default();
                        }
                        _ => {}
                    }
                }
                prefix_mapper.add(prefix_name, prefix_iri);
            }

            // SPARQL-style `BASE <IRI>` -- same semantics as `@base`.
            Rule::SparqlBase => {
                if let Some(iri_pair) = item
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::BracketedIri)
                    .and_then(|bracketed| bracketed.into_inner().next())
                {
                    prefix_mapper.set_base(iri_pair.as_str());
                }
            }

            // `@keywords a, is, of, true, false .` -- this engine always
            // recognizes exactly that fixed keyword set (see the grammar
            // comment on `Keywords`), so honoring the directive means
            // validating every declared word is a member of it; an unknown
            // word is a real error (matching real N3/EYE, which rejects
            // `@keywords` entries it doesn't understand) rather than being
            // silently ignored.
            Rule::Keywords => {
                const KNOWN_KEYWORDS: [&str; 5] = ["a", "is", "of", "true", "false"];
                for child in item.into_inner() {
                    if child.as_rule() == Rule::KeywordList {
                        for word_pair in child.into_inner() {
                            if word_pair.as_rule() == Rule::KeywordWord {
                                let word = word_pair.as_str();
                                if !KNOWN_KEYWORDS.contains(&word) {
                                    return Err(format!(
                                        "N3 parse error: unknown @keywords entry '{}' (supported: a, is, of, true, false)",
                                        word
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            Rule::TP => {
                content.extend(parse_tp(item.into_inner(), &prefix_mapper));
            }

            // `rule` wraps exactly one of `forward_rule` ("{body} => {head}")
            // or `backward_rule` ("{head} <= {body}"). Both grammar
            // productions still name their braces `Body`/`Head` according to
            // their *semantic* role (see the grammar comment), so the same
            // extraction logic below handles both without needing to know
            // which one it is.
            Rule::rule => {
                for variant in item.into_inner() {
                    // `<=>` biconditional sugar: "{A} <=> {B} ." means both
                    // "{A} => {B} ." and "{B} => {A} .". Both sides are plain
                    // `Head`s (TP+ groups); each becomes the *body* (as
                    // unnegated literals) for a rule per triple on the other
                    // side -- i.e. two ordinary rules generated from the one
                    // biconditional statement.
                    if variant.as_rule() == Rule::biconditional_rule {
                        let mut sides: Vec<Vec<Triple>> = Vec::new();
                        for part in variant.into_inner() {
                            if part.as_rule() == Rule::Head {
                                let mut side_triples = Vec::new();
                                for tp_pair in part.into_inner() {
                                    if tp_pair.as_rule() == Rule::TP {
                                        side_triples.extend(parse_tp(tp_pair.into_inner(), &prefix_mapper));
                                    }
                                }
                                sides.push(side_triples);
                            }
                        }
                        if sides.len() == 2 {
                            let left = sides[0].clone();
                            let right = sides[1].clone();
                            let left_body: Vec<BodyLiteral> = left
                                .iter()
                                .cloned()
                                .map(|pattern| BodyLiteral { negated: false, pattern })
                                .collect();
                            let right_body: Vec<BodyLiteral> = right
                                .iter()
                                .cloned()
                                .map(|pattern| BodyLiteral { negated: false, pattern })
                                .collect();
                            // left => right
                            for head in right.iter().cloned() {
                                rules.push(ReasonerRule { body: left_body.clone(), head });
                            }
                            // right => left
                            for head in left.iter().cloned() {
                                rules.push(ReasonerRule { body: right_body.clone(), head });
                            }
                        }
                        continue;
                    }

                    if variant.as_rule() != Rule::forward_rule && variant.as_rule() != Rule::backward_rule {
                        continue;
                    }

                    let mut body: Vec<BodyLiteral> = Vec::new();
                    let mut head_triples: Vec<Triple> = Vec::new();
                    let mut is_deny_head = false;

                    let parts: Vec<Pair<Rule>> = variant.into_inner().collect();

                    // The rule's antecedent (Body) is itself a formula per the
                    // N3 CG spec, so `@forAll`/`@forSome` may be declared
                    // directly inside its braces (e.g. `{ @forAll ?x . ?x a
                    // :Dog } => { ?x a :Mammal }.`), scoped to this rule (body
                    // + head). Push a fresh scope for the whole rule and
                    // pre-register any such declarations before parsing the
                    // body/head triples, matching the pre-pass pattern used
                    // for `Formula` and the document root.
                    with_new_scope(|| {
                        for part in &parts {
                            if part.as_rule() == Rule::Body {
                                for bl_pair in part.clone().into_inner() {
                                    if bl_pair.as_rule() == Rule::ForAll || bl_pair.as_rule() == Rule::ForSome {
                                        register_quantifier_declarations(&bl_pair);
                                    }
                                }
                            }
                        }

                        for part in parts {
                            match part.as_rule() {
                                Rule::Body => {
                                    for bl_pair in part.into_inner() {
                                        // Declarations were already registered
                                        // in the pre-pass above.
                                        if bl_pair.as_rule() == Rule::ForAll || bl_pair.as_rule() == Rule::ForSome {
                                            continue;
                                        }
                                        // `true` (TrueBody) means "unconditionally
                                        // true, no antecedent constraints" -- a
                                        // real EYE corpus idiom (e.g. the `peano`
                                        // case's `{(?A 0) :add ?A} <= true.`);
                                        // contributes zero body literals.
                                        if bl_pair.as_rule() == Rule::TrueBody {
                                            continue;
                                        }
                                        // bl_pair is a BodyLiteral
                                        let is_negated = bl_pair.as_str().trim_start().starts_with("not");
                                        // Find the TP inside the BodyLiteral
                                        let tp_pair = bl_pair
                                            .into_inner()
                                            .find(|p| p.as_rule() == Rule::TP)
                                            .expect("BodyLiteral must contain a TP");
                                        let patterns = parse_tp(tp_pair.into_inner(), &prefix_mapper);
                                        for pattern in patterns {
                                            body.push(BodyLiteral { negated: is_negated, pattern });
                                        }
                                    }
                                }
                                Rule::Head => {
                                    for tp_pair in part.into_inner() {
                                        if tp_pair.as_rule() == Rule::TP {
                                            head_triples.extend(parse_tp(tp_pair.into_inner(), &prefix_mapper));
                                        } else if tp_pair.as_rule() == Rule::DenyHead {
                                            is_deny_head = true;
                                        }
                                    }
                                }
                                Rule::EOI => {}
                                _ => {}
                            }
                        }
                    });

                    if is_deny_head {
                        rules.push(ReasonerRule::new_denial(body.clone()));
                    } else {
                        // Emit one rule per head triple (multi-head rules desugar to multiple rules)
                        for head in head_triples {
                            rules.push(ReasonerRule {
                                body: body.clone(),
                                head,
                            });
                        }
                    }
                }
            }

            Rule::EOI => {}
            _ => {}
        }
    }

    Ok((content, rules))
}

/// Parse an N3-rule string into a list of Datalog `Rule`s (discarding any
/// plain top-level fact triples -- use `parse_document` to get both).
pub fn parse(input: &str) -> Result<Vec<ReasonerRule>, String> {
    parse_document(input).map(|(_content, rules)| rules)
}

#[cfg(test)]
#[path = "n3rule_parser_test.rs"]
mod n3rule_parser_test;
