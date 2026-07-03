//! RDF-as-workflow front end — a bounded Turtle-subset parser whose only
//! output is a deterministic canonical form and its content address.
//!
//! ## Doctrine
//!
//! The graph *is* the law: the hash that anchors every downstream receipt is
//! computed from the parsed, canonicalized triples — never asserted by the
//! document, never fuzzy. Same triples in any surface order or whitespace
//! yield the same `graph_hash`; different exact bytes are still nameable via
//! a separate `ttl_hash` that is a receipt field only, never folded into any
//! chain. Every cap violation and every grammar violation is a typed
//! [`Refusal`] naming the culprit (line + column) — never a panic, never a
//! silent truncation.
//!
//! Lineage: knhk genesis-graph computed-hash-per-fired-rule + replay verifier
//! (genuine) -> imported; cns bitactor_compiler.py sorted-triple hashing
//! (partial) -> upgraded to ground-only canonical form; bitactor
//! asserted-spec-hash (anti-pattern) -> refused by name.
//!
//! Sections: `lex`, `parse`, `canon`, `vocab`, `ir`, `lower`, `run`, `replay`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use chatman_common::provenance::{content_address, fold_event, genesis_seed};

use crate::dag::{Dag, DagNode, MemoCache, NodeRunner, SupervisedReceipt};
use crate::datalog::{Atom as DlAtom, Program, Term};
use crate::geometry::FailureGeometry;
use crate::park::ParkManager;
use crate::sequence::{Capability, Constraint, SequencePlan, SequenceProblem, Solver};
use crate::solver8::Solver8;
use crate::supervise::{RestartPolicy, SupervisionTopology};
use crate::Refusal;

// ---------------------------------------------------------------------------
// caps
// ---------------------------------------------------------------------------

/// Hard cap on input document size in bytes.
pub const MAX_TTL_BYTES: usize = 65_536;
/// Hard cap on parsed triples.
pub const MAX_TRIPLES: usize = 4_096;
/// Hard cap on any fully-expanded IRI length in bytes.
pub const MAX_IRI_LEN: usize = 256;
/// Hard cap on any string literal length in bytes (unescaped).
pub const MAX_LIT_LEN: usize = 1_024;
/// Hard cap on declared prefixes.
pub const MAX_PREFIXES: usize = 32;

// ---------------------------------------------------------------------------
// core triple types
// ---------------------------------------------------------------------------

/// An object position value: IRI, string literal, or integer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Object {
    /// A fully-expanded IRI.
    Iri(String),
    /// A plain string literal (no language tag, no datatype — refused at parse).
    Str(String),
    /// A signed 64-bit integer literal.
    Int(i64),
}

/// One ground triple. Subject and predicate are fully-expanded IRIs —
/// prefixes are resolved at parse time; blank nodes are refused, so every
/// term is ground and the sorted rendering below is a sound canonical form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Triple {
    /// Subject IRI (fully expanded).
    pub s: String,
    /// Predicate IRI (fully expanded; `a` expands to `rdf:type`).
    pub p: String,
    /// Object term.
    pub o: Object,
}

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

// ---------------------------------------------------------------------------
// lex
// ---------------------------------------------------------------------------

/// Token kinds produced by the lexer. Positions are 1-based line/column of
/// the token's first byte — every refusal names its culprit.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    /// `@prefix`
    AtPrefix,
    /// `pn:local` — prefixed name (prefix part may be empty).
    Qname(String, String),
    /// A bare prefix declaration name `pn:` (qname with empty local is the
    /// same shape; disambiguated by parser context).
    Iri(String),
    /// String literal, unescaped.
    Str(String),
    /// Integer literal.
    Int(i64),
    /// `a` keyword.
    A,
    Dot,
    Semi,
    Comma,
}

#[derive(Debug, Clone)]
struct Spanned {
    tok: Tok,
    line: usize,
    column: usize,
}

fn malformed(line: usize, column: usize, detail: impl Into<String>) -> Refusal {
    Refusal::GraphMalformed {
        line,
        column,
        detail: detail.into(),
    }
}

fn cap(what: &str, cap: usize, actual: usize) -> Refusal {
    Refusal::GraphCapExceeded {
        what: what.to_string(),
        cap: cap as u64,
        actual: actual as u64,
    }
}

struct Lexer<'a> {
    bytes: &'a [u8],
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            bytes: src.as_bytes(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    fn skip_ws_and_comments(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.bump();
                }
                b'#' => {
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                _ => break,
            }
        }
    }

    fn lex_string(&mut self, line: usize, column: usize) -> Result<Tok, Refusal> {
        // opening quote already consumed
        let mut out = String::new();
        loop {
            let Some(b) = self.bump() else {
                return Err(malformed(line, column, "unterminated string literal"));
            };
            match b {
                b'"' => break,
                b'\\' => {
                    let (el, ec) = (self.line, self.col);
                    match self.bump() {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'n') => out.push('\n'),
                        Some(b't') => out.push('\t'),
                        Some(c) => {
                            return Err(malformed(
                                el,
                                ec,
                                format!("unsupported escape '\\{}'", c as char),
                            ))
                        }
                        None => return Err(malformed(line, column, "unterminated string literal")),
                    }
                }
                b'\n' | b'\r' => {
                    return Err(malformed(line, column, "raw newline in string literal"))
                }
                c if c < 0x20 => {
                    return Err(malformed(
                        line,
                        column,
                        format!("raw control character 0x{c:02x} in string literal"),
                    ))
                }
                c => {
                    // Reassemble UTF-8 byte-by-byte: source was &str, so bytes are valid.
                    let mut buf = vec![c];
                    while self
                        .peek()
                        .map(|n| n & 0xC0 == 0x80)
                        .unwrap_or(false)
                    {
                        buf.push(self.bump().expect("peeked continuation byte"));
                    }
                    out.push_str(
                        std::str::from_utf8(&buf)
                            .map_err(|_| malformed(line, column, "invalid UTF-8 in string"))?,
                    );
                }
            }
            if out.len() > MAX_LIT_LEN {
                return Err(cap("lit_len", MAX_LIT_LEN, out.len()));
            }
        }
        Ok(Tok::Str(out))
    }

    fn lex_iriref(&mut self, line: usize, column: usize) -> Result<Tok, Refusal> {
        // opening '<' consumed
        let mut out = String::new();
        loop {
            let Some(b) = self.bump() else {
                return Err(malformed(line, column, "unterminated IRIREF"));
            };
            match b {
                b'>' => break,
                b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'{' | b'}' | b'|' | b'^' | b'`' => {
                    return Err(malformed(
                        line,
                        column,
                        format!("illegal character '{}' in IRIREF", b as char),
                    ))
                }
                c if c < 0x20 => {
                    return Err(malformed(line, column, "control character in IRIREF"))
                }
                c => out.push(c as char),
            }
            if out.len() > MAX_IRI_LEN {
                return Err(cap("iri_len", MAX_IRI_LEN, out.len()));
            }
        }
        Ok(Tok::Iri(out))
    }

    fn next_token(&mut self) -> Result<Option<Spanned>, Refusal> {
        self.skip_ws_and_comments();
        let (line, column) = (self.line, self.col);
        let Some(b) = self.peek() else {
            return Ok(None);
        };
        let tok = match b {
            b'.' => {
                self.bump();
                Tok::Dot
            }
            b';' => {
                self.bump();
                Tok::Semi
            }
            b',' => {
                self.bump();
                Tok::Comma
            }
            b'"' => {
                self.bump();
                self.lex_string(line, column)?
            }
            b'<' => {
                self.bump();
                self.lex_iriref(line, column)?
            }
            b'[' | b']' => {
                return Err(malformed(line, column, "blank node '[]' is refused"));
            }
            b'(' | b')' => {
                return Err(malformed(line, column, "collection '()' is refused"));
            }
            b'@' => {
                self.bump();
                let word = self.lex_bareword();
                match word.as_str() {
                    "prefix" => Tok::AtPrefix,
                    "base" => return Err(malformed(line, column, "'@base' is refused")),
                    _ => {
                        return Err(malformed(
                            line,
                            column,
                            format!("unsupported directive or language tag '@{word}'"),
                        ))
                    }
                }
            }
            b'^' => {
                return Err(malformed(line, column, "'^^' datatype is refused"));
            }
            b'-' | b'0'..=b'9' => {
                let mut s = String::new();
                if b == b'-' {
                    s.push('-');
                    self.bump();
                }
                let mut digits = 0usize;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        s.push(c as char);
                        digits += 1;
                        self.bump();
                    } else {
                        break;
                    }
                }
                if digits == 0 {
                    return Err(malformed(line, column, "'-' without digits"));
                }
                if let Some(c) = self.peek() {
                    if c == b'.' && self.bytes.get(self.i + 1).is_some_and(u8::is_ascii_digit) {
                        return Err(malformed(line, column, "decimal literal is refused"));
                    }
                    if c == b'e' || c == b'E' {
                        return Err(malformed(line, column, "double literal is refused"));
                    }
                }
                let v: i64 = s.parse().map_err(|_| {
                    malformed(line, column, format!("integer '{s}' does not fit i64"))
                })?;
                Tok::Int(v)
            }
            b'_' => {
                return Err(malformed(line, column, "blank node '_:' is refused"));
            }
            _ => {
                let word = self.lex_bareword();
                if word.is_empty() {
                    return Err(malformed(
                        line,
                        column,
                        format!("unexpected character '{}'", b as char),
                    ));
                }
                if let Some(colon) = word.find(':') {
                    let (pn, local) = (word[..colon].to_string(), word[colon + 1..].to_string());
                    if word.as_str() == "true" || word.as_str() == "false" {
                        return Err(malformed(line, column, "boolean literal is refused"));
                    }
                    Tok::Qname(pn, local)
                } else if word == "a" {
                    Tok::A
                } else if word == "true" || word == "false" {
                    return Err(malformed(line, column, "boolean literal is refused"));
                } else {
                    return Err(malformed(
                        line,
                        column,
                        format!("bare word '{word}' is not a term"),
                    ));
                }
            }
        };
        Ok(Some(Spanned { tok, line, column }))
    }

    fn lex_bareword(&mut self) -> String {
        let mut out = String::new();
        while let Some(b) = self.peek() {
            let ok = b.is_ascii_alphanumeric() || matches!(b, b':' | b'_' | b'-' | b'?');
            if !ok {
                break;
            }
            out.push(b as char);
            self.bump();
        }
        out
    }
}

// ---------------------------------------------------------------------------
// parse
// ---------------------------------------------------------------------------

struct Parser {
    toks: Vec<Spanned>,
    i: usize,
    prefixes: Vec<(String, String)>,
    triples: Vec<Triple>,
}

impl Parser {
    fn peek(&self) -> Option<&Spanned> {
        self.toks.get(self.i)
    }

    fn bump(&mut self) -> Option<Spanned> {
        let t = self.toks.get(self.i).cloned();
        if t.is_some() {
            self.i += 1;
        }
        t
    }

    fn eof_err(&self) -> Refusal {
        let (line, column) = self
            .toks
            .last()
            .map(|t| (t.line, t.column))
            .unwrap_or((1, 1));
        malformed(line, column, "unexpected end of input")
    }

    fn expand(&self, pn: &str, local: &str, line: usize, column: usize) -> Result<String, Refusal> {
        let base = self
            .prefixes
            .iter()
            .rev()
            .find(|(p, _)| p == pn)
            .map(|(_, iri)| iri.clone())
            .ok_or_else(|| malformed(line, column, format!("undeclared prefix '{pn}:'")))?;
        let iri = format!("{base}{local}");
        if iri.len() > MAX_IRI_LEN {
            return Err(cap("iri_len", MAX_IRI_LEN, iri.len()));
        }
        Ok(iri)
    }

    fn parse_prefix(&mut self) -> Result<(), Refusal> {
        // '@prefix' consumed
        let name = self.bump().ok_or_else(|| self.eof_err())?;
        let pn = match name.tok {
            Tok::Qname(pn, local) if local.is_empty() => pn,
            _ => {
                return Err(malformed(
                    name.line,
                    name.column,
                    "expected 'name:' after @prefix",
                ))
            }
        };
        let iri = self.bump().ok_or_else(|| self.eof_err())?;
        let Tok::Iri(base) = iri.tok else {
            return Err(malformed(iri.line, iri.column, "expected <IRI> in @prefix"));
        };
        let dot = self.bump().ok_or_else(|| self.eof_err())?;
        if dot.tok != Tok::Dot {
            return Err(malformed(dot.line, dot.column, "expected '.' after @prefix"));
        }
        self.prefixes.push((pn, base));
        if self.prefixes.len() > MAX_PREFIXES {
            return Err(cap("prefixes", MAX_PREFIXES, self.prefixes.len()));
        }
        Ok(())
    }

    fn parse_term_iri(&mut self, what: &str) -> Result<String, Refusal> {
        let t = self.bump().ok_or_else(|| self.eof_err())?;
        match t.tok {
            Tok::Iri(iri) => {
                if iri.len() > MAX_IRI_LEN {
                    return Err(cap("iri_len", MAX_IRI_LEN, iri.len()));
                }
                Ok(iri)
            }
            Tok::Qname(pn, local) => self.expand(&pn, &local, t.line, t.column),
            _ => Err(malformed(
                t.line,
                t.column,
                format!("expected IRI or prefixed name as {what}"),
            )),
        }
    }

    fn parse_object(&mut self) -> Result<Object, Refusal> {
        let t = self.bump().ok_or_else(|| self.eof_err())?;
        match t.tok {
            Tok::Iri(iri) => {
                if iri.len() > MAX_IRI_LEN {
                    return Err(cap("iri_len", MAX_IRI_LEN, iri.len()));
                }
                Ok(Object::Iri(iri))
            }
            Tok::Qname(pn, local) => Ok(Object::Iri(self.expand(&pn, &local, t.line, t.column)?)),
            Tok::Str(s) => Ok(Object::Str(s)),
            Tok::Int(v) => Ok(Object::Int(v)),
            _ => Err(malformed(t.line, t.column, "expected object term")),
        }
    }

    fn push_triple(&mut self, s: String, p: String, o: Object) -> Result<(), Refusal> {
        self.triples.push(Triple { s, p, o });
        if self.triples.len() > MAX_TRIPLES {
            return Err(cap("triples", MAX_TRIPLES, self.triples.len()));
        }
        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<(), Refusal> {
        let subject = self.parse_term_iri("subject")?;
        loop {
            // predicate
            let pred = {
                let t = self.peek().cloned().ok_or_else(|| self.eof_err())?;
                match t.tok {
                    Tok::A => {
                        self.bump();
                        RDF_TYPE.to_string()
                    }
                    _ => self.parse_term_iri("predicate")?,
                }
            };
            // object list
            loop {
                let o = self.parse_object()?;
                self.push_triple(subject.clone(), pred.clone(), o)?;
                match self.peek() {
                    Some(t) if t.tok == Tok::Comma => {
                        self.bump();
                    }
                    _ => break,
                }
            }
            let t = self.bump().ok_or_else(|| self.eof_err())?;
            match t.tok {
                Tok::Dot => return Ok(()),
                Tok::Semi => {
                    // allow trailing ';' before '.'
                    if let Some(n) = self.peek() {
                        if n.tok == Tok::Dot {
                            self.bump();
                            return Ok(());
                        }
                    }
                    continue;
                }
                _ => {
                    return Err(malformed(
                        t.line,
                        t.column,
                        "expected '.', ';' or ',' after object",
                    ))
                }
            }
        }
    }
}

/// Parse the bounded Turtle subset into fully-expanded ground triples.
///
/// Every grammar or subset violation is a [`Refusal::GraphMalformed`] naming
/// line and column; every cap violation is a [`Refusal::GraphCapExceeded`].
/// Never panics on any input.
pub fn parse_ttl(src: &str) -> Result<Vec<Triple>, Refusal> {
    if src.len() > MAX_TTL_BYTES {
        return Err(cap("ttl_bytes", MAX_TTL_BYTES, src.len()));
    }
    let mut lexer = Lexer::new(src);
    let mut toks = Vec::new();
    while let Some(t) = lexer.next_token()? {
        toks.push(t);
    }
    let mut parser = Parser {
        toks,
        i: 0,
        prefixes: Vec::new(),
        triples: Vec::new(),
    };
    while let Some(t) = parser.peek() {
        if t.tok == Tok::AtPrefix {
            parser.bump();
            parser.parse_prefix()?;
        } else {
            parser.parse_stmt()?;
        }
    }
    Ok(parser.triples)
}

// ---------------------------------------------------------------------------
// canon
// ---------------------------------------------------------------------------

fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn render_object(o: &Object) -> String {
    match o {
        Object::Iri(iri) => format!("<{iri}>"),
        Object::Str(s) => format!("\"{}\"", escape_str(s)),
        Object::Int(v) => format!("{v}"), // shortest-form decimal; i64 Display has no leading zeros, -0 impossible
    }
}

/// Render triples in the exact canonical form: one N-Triples-style line per
/// triple, byte-sorted, deduplicated, newline-joined with a trailing newline.
///
/// Sound as a canonical form because blank nodes are refused at parse — every
/// term is ground, so sorted rendering is a total order on graphs.
pub fn canonical_form(triples: &[Triple]) -> String {
    let mut lines: Vec<String> = triples
        .iter()
        .map(|t| format!("<{}> <{}> {} .", t.s, t.p, render_object(&t.o)))
        .collect();
    lines.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    lines.dedup();
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Content address of the canonical form: the graph's law-hash. Computed,
/// never asserted (the bitactor asserted-spec-hash anti-pattern is refused
/// by the vocabulary layer).
#[must_use]
pub fn graph_hash(triples: &[Triple]) -> String {
    chatman_common::provenance::content_address(canonical_form(triples).as_bytes())
}

/// Content address of the exact raw bytes of the source document. A receipt
/// field only — never folded into any chain (reformatting the document
/// changes `ttl_hash` but not `graph_hash`).
#[must_use]
pub fn ttl_hash(src: &str) -> String {
    chatman_common::provenance::content_address(src.as_bytes())
}

// ---------------------------------------------------------------------------
// vocab
// ---------------------------------------------------------------------------

/// The workflow vocabulary namespace. Closed world: every predicate in this
/// namespace must appear in the vocabulary table, else the triple is a typed
/// [`Refusal::UnknownPredicate`]. Foreign-namespace triples are ignored
/// semantically but still canonicalized and hashed.
pub const WF_NS: &str = "http://seanchatmangpt.github.io/praxis/workflow#";

const WF_CLASSES: [&str; 4] = ["Workflow", "Capability", "Atom", "Constraint"];
const WF_PREDICATES: [&str; 26] = [
    "budget", "init", "goal", "name", "params", "cost", "pre", "add", "del", "predicate", "arg0",
    "arg1", "arg2", "arg3", "arg4", "arg5", "arg6", "arg7", "kind", "a", "b", "k", "handler",
    "delegability", "capability", "constraint",
];

fn ill(subject: &str, detail: impl Into<String>) -> Refusal {
    Refusal::WorkflowIllFormed {
        subject: subject.to_string(),
        detail: detail.into(),
    }
}

/// Per-subject view of the graph, in canonical (sorted) triple order.
struct NodeIndex<'a> {
    by_subject: BTreeMap<&'a str, Vec<(&'a str, &'a Object)>>,
}

impl<'a> NodeIndex<'a> {
    fn build(triples: &'a [Triple]) -> Result<Self, Refusal> {
        let mut sorted: Vec<&Triple> = triples.iter().collect();
        sorted.sort_unstable();
        sorted.dedup();
        let mut by_subject: BTreeMap<&str, Vec<(&str, &Object)>> = BTreeMap::new();
        for t in sorted {
            // Closed-world vocabulary enforcement. The bitactor anti-pattern —
            // asserting the hash the receipt should compute — is refused by
            // name: no `wf:` local containing "hash" is in the table.
            if let Some(local) = t.p.strip_prefix(WF_NS) {
                if !WF_PREDICATES.contains(&local) {
                    return Err(Refusal::UnknownPredicate {
                        predicate: t.p.clone(),
                        subject: t.s.clone(),
                    });
                }
            }
            if t.p == RDF_TYPE {
                if let Object::Iri(class) = &t.o {
                    if let Some(local) = class.strip_prefix(WF_NS) {
                        if !WF_CLASSES.contains(&local) {
                            return Err(ill(&t.s, format!("unknown wf: class '{local}'")));
                        }
                    }
                }
            }
            by_subject.entry(&t.s).or_default().push((&t.p, &t.o));
        }
        Ok(Self { by_subject })
    }

    fn objects(&self, subject: &str, local: &str) -> Vec<&'a Object> {
        let pred = format!("{WF_NS}{local}");
        self.by_subject
            .get(subject)
            .map(|props| {
                props
                    .iter()
                    .filter(|(p, _)| *p == pred)
                    .map(|(_, o)| *o)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn subjects_of_class(&self, class_local: &str) -> Vec<&'a str> {
        let class = format!("{WF_NS}{class_local}");
        self.by_subject
            .iter()
            .filter(|(_, props)| {
                props
                    .iter()
                    .any(|(p, o)| *p == RDF_TYPE && matches!(o, Object::Iri(c) if *c == class))
            })
            .map(|(s, _)| *s)
            .collect()
    }

    fn one_str(&self, subject: &str, local: &str) -> Result<String, Refusal> {
        match self.objects(subject, local).as_slice() {
            [Object::Str(s)] => Ok(s.clone()),
            [] => Err(ill(subject, format!("missing wf:{local}"))),
            [_] => Err(ill(subject, format!("wf:{local} must be a string literal"))),
            _ => Err(ill(subject, format!("multiple wf:{local}"))),
        }
    }

    fn one_int(&self, subject: &str, local: &str) -> Result<i64, Refusal> {
        match self.objects(subject, local).as_slice() {
            [Object::Int(v)] => Ok(*v),
            [] => Err(ill(subject, format!("missing wf:{local}"))),
            [_] => Err(ill(subject, format!("wf:{local} must be an integer literal"))),
            _ => Err(ill(subject, format!("multiple wf:{local}"))),
        }
    }

    fn opt_str(&self, subject: &str, local: &str) -> Result<Option<String>, Refusal> {
        match self.objects(subject, local).as_slice() {
            [] => Ok(None),
            [Object::Str(s)] => Ok(Some(s.clone())),
            [_] => Err(ill(subject, format!("wf:{local} must be a string literal"))),
            _ => Err(ill(subject, format!("multiple wf:{local}"))),
        }
    }

    fn opt_int(&self, subject: &str, local: &str) -> Result<Option<i64>, Refusal> {
        match self.objects(subject, local).as_slice() {
            [] => Ok(None),
            [Object::Int(v)] => Ok(Some(*v)),
            [_] => Err(ill(subject, format!("wf:{local} must be an integer literal"))),
            _ => Err(ill(subject, format!("multiple wf:{local}"))),
        }
    }

    /// Resolve every `wf:{local}` object as an Atom IRI reference.
    fn atom_refs(&self, subject: &str, local: &str) -> Result<Vec<&'a str>, Refusal> {
        self.objects(subject, local)
            .into_iter()
            .map(|o| match o {
                Object::Iri(iri) => Ok(iri.as_str()),
                _ => Err(ill(subject, format!("wf:{local} must reference an atom IRI"))),
            })
            .collect()
    }
}

/// Closed-world vocabulary check over the `wf:` namespace, exposed for the
/// admission gate: a post-state graph whose `wf:` triples violate the closed
/// vocabulary must be refused *at admission*, not later at extraction.
pub(crate) fn vocab_check(triples: &[Triple]) -> Result<(), Refusal> {
    NodeIndex::build(triples).map(|_| ())
}

// ---------------------------------------------------------------------------
// ir
// ---------------------------------------------------------------------------

/// One IR atom: a predicate name plus argument strings. Arguments `"?0"`
/// through `"?7"` denote `Term::Var(0..=7)`; anything else is a constant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IrAtom {
    /// Predicate name.
    pub predicate: String,
    /// Argument strings (`"?0".."?7"` = variables, else constants).
    pub args: Vec<String>,
}

/// One IR capability: name, parameter count, cost, and pre/add/del atoms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrCapability {
    /// Unique capability name.
    pub name: String,
    /// Number of parameter variables (0..=8).
    pub params: u8,
    /// Non-negative cost.
    pub cost: u32,
    /// Precondition atoms.
    pub pre: Vec<IrAtom>,
    /// Atoms added on execution.
    pub add: Vec<IrAtom>,
    /// Atoms deleted on execution.
    pub del: Vec<IrAtom>,
}

/// One IR constraint: uniform `kind` string plus optional `a`/`b`/`k` fields.
/// Kinds and required fields mirror [`crate::sequence::Constraint`] exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrConstraint {
    /// Constraint kind: `before` | `after` | `not-later` | `not-earlier` |
    /// `excludes` | `requires` | `at-most` | `budget`.
    pub kind: String,
    /// First capability name (when the kind requires one).
    pub a: Option<String>,
    /// Second capability name (when the kind requires one).
    pub b: Option<String>,
    /// Numeric bound (step index, occurrence count, or cost budget).
    pub k: Option<u32>,
}

impl IrConstraint {
    fn render(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.kind,
            self.a.as_deref().unwrap_or(""),
            self.b.as_deref().unwrap_or(""),
            self.k.map(|k| k.to_string()).unwrap_or_default(),
        )
    }
}

/// The extracted workflow IR. Every collection is a sorted `Vec` (no maps,
/// no floats, no timestamps), so `serde_json::to_string` is deterministic
/// and [`ir_hash`] is a sound content address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowIr {
    /// Solve horizon and step budget, 1..=8.
    pub budget: u8,
    /// Initial ground atoms.
    pub init: Vec<IrAtom>,
    /// Goal atoms.
    pub goal: Vec<IrAtom>,
    /// Capabilities, sorted by name.
    pub capabilities: Vec<IrCapability>,
    /// Constraints, sorted by render string.
    pub constraints: Vec<IrConstraint>,
}

fn var_index(arg: &str) -> Option<u8> {
    let rest = arg.strip_prefix('?')?;
    if rest.len() == 1 {
        let d = rest.as_bytes()[0];
        if (b'0'..=b'7').contains(&d) {
            return Some(d - b'0');
        }
    }
    None
}

/// Extract one atom node: `wf:predicate` required, `wf:arg0..wf:arg7`
/// contiguous from 0 (a gap is [`Refusal::WorkflowIllFormed`]).
fn extract_atom(idx: &NodeIndex<'_>, iri: &str) -> Result<IrAtom, Refusal> {
    if !idx.subjects_of_class("Atom").contains(&iri) {
        return Err(ill(iri, "referenced node is not declared 'a wf:Atom'"));
    }
    let predicate = idx.one_str(iri, "predicate")?;
    let mut args = Vec::new();
    let mut ended = false;
    for i in 0..8u8 {
        match idx.opt_str(iri, &format!("arg{i}"))? {
            Some(v) if !ended => args.push(v),
            Some(_) => {
                return Err(ill(
                    iri,
                    format!("wf:arg{i} present but wf:arg{} missing (argument gap)", i - 1),
                ))
            }
            None => ended = true,
        }
    }
    Ok(IrAtom { predicate, args })
}

fn extract_constraint(
    idx: &NodeIndex<'_>,
    iri: &str,
    cap_names: &BTreeSet<&str>,
) -> Result<IrConstraint, Refusal> {
    let kind = idx.one_str(iri, "kind")?;
    let a = idx.opt_str(iri, "a")?;
    let b = idx.opt_str(iri, "b")?;
    let k = match idx.opt_int(iri, "k")? {
        None => None,
        Some(v) if (0..=i64::from(u32::MAX)).contains(&v) =>
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Some(v as u32)
        }
        Some(v) => return Err(ill(iri, format!("wf:k {v} out of range 0..=u32::MAX"))),
    };
    let need_a = |a: &Option<String>| -> Result<(), Refusal> {
        if a.is_none() {
            return Err(ill(iri, format!("kind '{kind}' requires wf:a")));
        }
        Ok(())
    };
    let need_k_u8 = |k: Option<u32>| -> Result<(), Refusal> {
        match k {
            Some(v) if v <= 255 => Ok(()),
            Some(v) => Err(ill(iri, format!("kind '{kind}' requires wf:k <= 255, got {v}"))),
            None => Err(ill(iri, format!("kind '{kind}' requires wf:k"))),
        }
    };
    match kind.as_str() {
        "before" | "after" | "excludes" | "requires" => {
            need_a(&a)?;
            if b.is_none() {
                return Err(ill(iri, format!("kind '{kind}' requires wf:b")));
            }
        }
        "not-later" | "not-earlier" | "at-most" => {
            need_a(&a)?;
            need_k_u8(k)?;
        }
        "budget" => {
            if k.is_none() {
                return Err(ill(iri, format!("kind '{kind}' requires wf:k")));
            }
        }
        other => return Err(ill(iri, format!("unknown constraint kind '{other}'"))),
    }
    for name in [&a, &b].into_iter().flatten() {
        if !cap_names.contains(name.as_str()) {
            return Err(ill(
                iri,
                format!("constraint names undeclared capability '{name}'"),
            ));
        }
    }
    Ok(IrConstraint { kind, a, b, k })
}

/// Extract the workflow IR from parsed triples. Closed-world over the `wf:`
/// namespace; every shape violation is a typed [`Refusal`] naming the
/// subject; `wf:budget > 8` reuses [`Refusal::BudgetExceeded`].
pub fn extract_ir(triples: &[Triple]) -> Result<WorkflowIr, Refusal> {
    let idx = NodeIndex::build(triples)?;

    // Exactly one wf:Workflow node.
    let workflows = idx.subjects_of_class("Workflow");
    let wf = match workflows.as_slice() {
        [one] => *one,
        [] => return Err(ill("(document)", "no 'a wf:Workflow' node")),
        many => {
            return Err(ill(
                many[0],
                format!("{} wf:Workflow nodes; exactly one required", many.len()),
            ))
        }
    };

    // Budget: 1..=8; over-budget is the existing typed refusal.
    let budget = idx.one_int(wf, "budget")?;
    if budget > 8 {
        #[allow(clippy::cast_sign_loss)]
        return Err(Refusal::BudgetExceeded {
            what: "wf:budget".to_string(),
            budget: 8,
            spent: budget as u64,
            salvage: "declare wf:budget <= 8".to_string(),
        });
    }
    if budget < 1 {
        return Err(ill(wf, format!("wf:budget {budget} out of range 1..=8")));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let budget = budget as u8;

    // Init atoms: repeatable, ground only.
    let mut init = Vec::new();
    for iri in idx.atom_refs(wf, "init")? {
        let atom = extract_atom(&idx, iri)?;
        if let Some(v) = atom.args.iter().find(|a| var_index(a).is_some()) {
            return Err(ill(iri, format!("wf:init atom has variable '{v}'; init must be ground")));
        }
        init.push(atom);
    }
    init.sort_unstable();

    // Goal atoms: one or more.
    let goal_refs = idx.atom_refs(wf, "goal")?;
    if goal_refs.is_empty() {
        return Err(ill(wf, "missing wf:goal (one or more required)"));
    }
    let mut goal = goal_refs
        .into_iter()
        .map(|iri| extract_atom(&idx, iri))
        .collect::<Result<Vec<_>, _>>()?;
    goal.sort_unstable();

    // Capabilities, sorted by unique name.
    let mut capabilities = Vec::new();
    for subject in idx.subjects_of_class("Capability") {
        let name = idx.one_str(subject, "name")?;
        let params = idx.one_int(subject, "params")?;
        if !(0..=8).contains(&params) {
            return Err(ill(subject, format!("wf:params {params} out of range 0..=8")));
        }
        let cost = idx.one_int(subject, "cost")?;
        if !(0..=i64::from(u32::MAX)).contains(&cost) {
            return Err(ill(subject, format!("wf:cost {cost} out of range 0..=u32::MAX")));
        }
        let mut lists = [Vec::new(), Vec::new(), Vec::new()];
        for (slot, local) in lists.iter_mut().zip(["pre", "add", "del"]) {
            for iri in idx.atom_refs(subject, local)? {
                slot.push(extract_atom(&idx, iri)?);
            }
            slot.sort_unstable();
        }
        let [pre, add, del] = lists;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        capabilities.push(IrCapability {
            name,
            params: params as u8,
            cost: cost as u32,
            pre,
            add,
            del,
        });
    }
    capabilities.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    for pair in capabilities.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(ill(wf, format!("duplicate capability name '{}'", pair[0].name)));
        }
    }
    let cap_names: BTreeSet<&str> = capabilities.iter().map(|c| c.name.as_str()).collect();

    // Constraints, sorted by render string.
    let mut constraints = idx
        .subjects_of_class("Constraint")
        .into_iter()
        .map(|iri| extract_constraint(&idx, iri, &cap_names))
        .collect::<Result<Vec<_>, _>>()?;
    constraints.sort_unstable_by_key(IrConstraint::render);

    Ok(WorkflowIr {
        budget,
        init,
        goal,
        capabilities,
        constraints,
    })
}

/// Content address of the IR's canonical JSON rendering. Deterministic
/// because every `WorkflowIr` collection is a sorted `Vec`.
pub fn ir_hash(ir: &WorkflowIr) -> Result<String, Refusal> {
    let json = serde_json::to_string(ir).map_err(|e| Refusal::InvalidInput {
        detail: format!("IR serialization failed: {e}"),
    })?;
    Ok(chatman_common::provenance::content_address(json.as_bytes()))
}

// ---------------------------------------------------------------------------
// lower
// ---------------------------------------------------------------------------

fn lower_atom(program: &mut Program, atom: &IrAtom) -> DlAtom {
    let pred = program.intern(&atom.predicate);
    let args = atom
        .args
        .iter()
        .map(|a| match var_index(a) {
            Some(v) => Term::Var(v),
            None => Term::Const(program.intern(a)),
        })
        .collect();
    DlAtom::new(pred, args)
}

fn lower_constraint(c: &IrConstraint) -> Constraint {
    // Totality over the 8 kinds is guaranteed by `extract_constraint`; the
    // required fields were validated there, so the unwraps below cannot
    // fire on any IR produced by `extract_ir`.
    let a = || c.a.clone().unwrap_or_default();
    let b = || c.b.clone().unwrap_or_default();
    #[allow(clippy::cast_possible_truncation)]
    let k8 = || c.k.unwrap_or_default() as u8;
    match c.kind.as_str() {
        "before" => Constraint::Before { a: a(), b: b() },
        "after" => Constraint::After { a: a(), b: b() },
        "excludes" => Constraint::Excludes { a: a(), b: b() },
        "requires" => Constraint::Requires { a: a(), b: b() },
        "not-later" => Constraint::NotLater { a: a(), k: k8() },
        "not-earlier" => Constraint::NotEarlier { a: a(), k: k8() },
        "at-most" => Constraint::AtMost { a: a(), n: k8() },
        _ => Constraint::Budget {
            max: c.k.unwrap_or_default(),
        },
    }
}

/// Lower the IR into a saturated [`Program`] plus a [`SequenceProblem`]
/// (budget doubles as the solve horizon). Downstream refusals from
/// [`SequenceProblem::with_constraints`] pass through unchanged.
pub fn lower(ir: &WorkflowIr) -> Result<(Program, SequenceProblem), Refusal> {
    let mut program = Program::new();
    for atom in &ir.init {
        let pred = program.intern(&atom.predicate);
        let syms: Vec<_> = atom.args.iter().map(|a| program.intern(a)).collect();
        program.add_fact(pred, &syms)?;
    }
    let caps: Vec<Capability> = ir
        .capabilities
        .iter()
        .map(|c| Capability {
            name: c.name.clone(),
            params: c.params,
            pre: c.pre.iter().map(|a| lower_atom(&mut program, a)).collect(),
            add: c.add.iter().map(|a| lower_atom(&mut program, a)).collect(),
            del: c.del.iter().map(|a| lower_atom(&mut program, a)).collect(),
            cost: c.cost,
        })
        .collect();
    let goal: Vec<DlAtom> = ir.goal.iter().map(|a| lower_atom(&mut program, a)).collect();
    let constraints: Vec<Constraint> = ir.constraints.iter().map(lower_constraint).collect();
    program.saturate()?;
    let problem = SequenceProblem::with_constraints(
        &program,
        caps,
        goal,
        ir.budget as usize,
        constraints,
    )?;
    Ok((program, problem))
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

/// Domain-separation tag for the workflow receipt chain.
const WORKFLOW_CHAIN_DOMAIN: &str = "praxis:workflow:v1";

/// Private deterministic runner: output = content address of the node id
/// plus its input hashes, ticks = 1. Never crashes — the graph front end
/// exercises the derived supervised path, not fault injection.
#[derive(Debug, Default, Clone, Copy)]
struct DeterministicRunner;

impl NodeRunner for DeterministicRunner {
    fn run(&mut self, node: &DagNode, inputs: &[Vec<u8>]) -> Vec<u8> {
        let mut frame = node.id.clone();
        for input in inputs {
            frame.push('\n');
            frame.push_str(&content_address(input));
        }
        content_address(frame.as_bytes()).into_bytes()
    }
}

/// The full derived-chain receipt for one TTL-defined workflow run.
///
/// Every hash below `ttl_hash` is *computed* from the previous stage —
/// nothing is asserted by the document. `ttl_hash` names the exact input
/// bytes but is a field only: it is never folded into `chain`, so a
/// reformatted document with the same triples yields the identical chain
/// while the exact bytes remain nameable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowReceipt {
    /// Content address of the raw TTL bytes (field only, never folded).
    pub ttl_hash: String,
    /// Content address of the canonical form of the parsed triples.
    pub graph_hash: String,
    /// Content address of the extracted [`WorkflowIr`] (sorted, serde-rendered).
    pub ir_hash: String,
    /// The solver's plan hash (`plan.receipt.plan_hash`).
    pub plan_hash: String,
    /// The derived supervision topology hash.
    pub topology_hash: String,
    /// The derived failure geometry hash.
    pub geometry_hash: String,
    /// Content address of the serde rendering of the supervised receipt.
    pub exec_hash: String,
    /// Fold of the six computed stage hashes over the workflow genesis seed,
    /// graph first — the graph is the law.
    pub chain: String,
    /// The solved plan (steps, cost, solve receipt).
    pub plan: SequencePlan,
    /// The supervised execution receipt (crash chain, dispositions, outcome).
    pub supervised: SupervisedReceipt,
}

fn exec_hash_of(supervised: &SupervisedReceipt) -> Result<String, Refusal> {
    let json = serde_json::to_string(supervised).map_err(|e| Refusal::InvalidInput {
        detail: format!("supervised receipt failed to serialize: {e}"),
    })?;
    Ok(content_address(json.as_bytes()))
}

fn fold_chain(
    graph_hash: &str,
    ir_hash: &str,
    plan_hash: &str,
    topology_hash: &str,
    geometry_hash: &str,
    exec_hash: &str,
) -> String {
    let mut chain = genesis_seed(WORKFLOW_CHAIN_DOMAIN);
    chain = fold_event(&chain, graph_hash.as_bytes());
    chain = fold_event(&chain, ir_hash.as_bytes());
    chain = fold_event(&chain, plan_hash.as_bytes());
    chain = fold_event(&chain, topology_hash.as_bytes());
    chain = fold_event(&chain, geometry_hash.as_bytes());
    chain = fold_event(&chain, exec_hash.as_bytes());
    chain
}

/// Execute a TTL-defined workflow end to end: parse → canonicalize →
/// extract IR → lower → solve ([`Solver8`]) → derive topology → derive
/// geometry → derive DAG → supervised execution. Returns the receipt whose
/// `chain` folds `graph_hash` first, then each derived stage in order.
/// Every failure on any path is a typed [`Refusal`]; nothing panics.
#[deprecated(since = "26.7.2", note = "use RiceQuarantine and Admission instead")]
pub fn execute_workflow(ttl: &str) -> Result<WorkflowReceipt, Refusal> {
    #[allow(deprecated)]
    execute_workflow_with(ttl, &mut DeterministicRunner)
}

/// [`execute_workflow`] with a caller-injected [`FallibleRunner`] — the
/// additive seam the hook-firing layer uses to dispatch graph-declared
/// handlers (which may lawfully crash). Every infallible [`NodeRunner`] is
/// blanket-adapted. The default path (`execute_workflow`) injects the
/// private deterministic runner and is byte-identical to the pre-seam
/// behavior.
#[deprecated(since = "26.7.2", note = "use RiceQuarantine and Admission instead")]
pub fn execute_workflow_with(
    ttl: &str,
    runner: &mut dyn crate::dag::FallibleRunner,
) -> Result<WorkflowReceipt, Refusal> {
    let triples = parse_ttl(ttl)?;
    execute_triples_with(&triples, ttl_hash(ttl), runner)
}

/// Execute a workflow already in triple form (the grounding path: a fired
/// hook's action fragment is a restriction of the ADMITTED graph — there
/// are no surface bytes). `ttl_hash` is set to the canonical form's content
/// address: the canonical bytes ARE the exact input here.
pub(crate) fn execute_from_triples(triples: &[Triple]) -> Result<WorkflowReceipt, Refusal> {
    let surface = content_address(canonical_form(triples).as_bytes());
    execute_triples_with(triples, surface, &mut DeterministicRunner)
}

fn execute_triples_with(
    triples: &[Triple],
    ttl_hash: String,
    runner: &mut dyn crate::dag::FallibleRunner,
) -> Result<WorkflowReceipt, Refusal> {
    let graph_hash = graph_hash(triples);
    let ir = extract_ir(triples)?;
    let ir_hash = ir_hash(&ir)?;
    let (_program, problem) = lower(&ir)?;
    let plan = Solver8.solve(&problem)?;
    let topology =
        SupervisionTopology::derive(&plan, &problem, RestartPolicy::new(1, 8)?)?;
    let geometry = FailureGeometry::derive(&topology, &plan, &problem);
    let dag = Dag::from_plan(&plan, &problem);
    let supervised = dag.execute_supervised(
        &topology,
        &geometry,
        runner,
        &mut MemoCache::new(),
        &mut ParkManager::new(),
        None,
        0,
    )?;
    let exec_hash = exec_hash_of(&supervised)?;
    let chain = fold_chain(
        &graph_hash,
        &ir_hash,
        &plan.receipt.plan_hash,
        &topology.topology_hash,
        &geometry.geometry_hash,
        &exec_hash,
    );
    Ok(WorkflowReceipt {
        ttl_hash,
        graph_hash,
        ir_hash,
        plan_hash: plan.receipt.plan_hash.clone(),
        topology_hash: topology.topology_hash.clone(),
        geometry_hash: geometry.geometry_hash.clone(),
        exec_hash,
        chain,
        plan,
        supervised,
    })
}

// ---------------------------------------------------------------------------
// replay
// ---------------------------------------------------------------------------

/// Independently re-derive every folded stage from `ttl` and compare the
/// result against `receipt`, field by field, in fold order. The first
/// divergent stage is named in [`Refusal::VerificationFailed`]. `ttl_hash`
/// is deliberately *not* compared: it is never folded, so an honest receipt
/// replays cleanly from any reformat of the same triples.
///
/// A receipt cannot vouch for itself — the bitactor asserted-spec-hash
/// anti-pattern — so replay recomputes; it never trusts.
pub fn replay_workflow(receipt: &WorkflowReceipt, ttl: &str) -> Result<(), Refusal> {
    let rederived = execute_workflow(ttl)?;
    let stages: [(&str, &str, &str); 7] = [
        ("graph_hash", &rederived.graph_hash, &receipt.graph_hash),
        ("ir_hash", &rederived.ir_hash, &receipt.ir_hash),
        ("plan_hash", &rederived.plan_hash, &receipt.plan_hash),
        ("topology_hash", &rederived.topology_hash, &receipt.topology_hash),
        ("geometry_hash", &rederived.geometry_hash, &receipt.geometry_hash),
        ("exec_hash", &rederived.exec_hash, &receipt.exec_hash),
        ("chain", &rederived.chain, &receipt.chain),
    ];
    for (name, computed, claimed) in stages {
        if computed != claimed {
            return Err(Refusal::VerificationFailed {
                failed: vec![name.to_string()],
            });
        }
    }
    // Bind the embedded payloads to the hashes just verified — a receipt
    // whose hash fields are honest but whose `plan`/`supervised` bodies are
    // forged must not pass replay (a consumer reading the bodies instead of
    // the hashes would otherwise be deceived).
    let computed_plan_hash = crate::sequence::plan_hash_of(&receipt.plan.steps);
    if computed_plan_hash != receipt.plan_hash || receipt.plan.receipt.plan_hash != receipt.plan_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["plan payload".to_string()],
        });
    }
    if exec_hash_of(&receipt.supervised)? != receipt.exec_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["supervised payload".to_string()],
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const DEMO: &str = r#"
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
@prefix ex: <http://example.org/> .
ex:pipeline a wf:Workflow ;
    wf:budget 3 ;
    wf:goal ex:goal0 .
ex:goal0 a wf:Atom ;
    wf:predicate "receipted" ;
    wf:arg0 "evidence" .
"#;

    #[test]
    fn same_bytes_same_hash() {
        let a = parse_ttl(DEMO).expect("parse");
        let b = parse_ttl(DEMO).expect("parse");
        assert_eq!(graph_hash(&a), graph_hash(&b));
        assert_eq!(canonical_form(&a), canonical_form(&b));
    }

    #[test]
    fn whitespace_and_order_invariant_hash() {
        let reordered = r#"
@prefix ex: <http://example.org/> .
@prefix wf: <http://seanchatmangpt.github.io/praxis/workflow#> .
ex:goal0 wf:arg0 "evidence" .
ex:goal0    wf:predicate   "receipted" .
ex:goal0 a wf:Atom .
ex:pipeline wf:goal ex:goal0 . ex:pipeline wf:budget 3 .
ex:pipeline a wf:Workflow .
"#;
        let a = parse_ttl(DEMO).expect("parse");
        let b = parse_ttl(reordered).expect("parse");
        assert_eq!(graph_hash(&a), graph_hash(&b));
    }

    #[test]
    fn ttl_hash_differs_when_bytes_differ_but_graph_hash_matches() {
        let reformatted = DEMO.replace("    ", "\t");
        let a = parse_ttl(DEMO).expect("parse");
        let b = parse_ttl(&reformatted).expect("parse");
        assert_eq!(graph_hash(&a), graph_hash(&b));
        assert_ne!(ttl_hash(DEMO), ttl_hash(&reformatted));
    }

    #[test]
    fn duplicate_triples_dedup_in_canonical_form() {
        let dup = "@prefix ex: <http://example.org/> .\nex:a ex:p ex:b .\nex:a ex:p ex:b .\n";
        let t = parse_ttl(dup).expect("parse");
        assert_eq!(t.len(), 2);
        assert_eq!(canonical_form(&t).lines().count(), 1);
    }

    #[test]
    fn a_keyword_expands_to_rdf_type() {
        let t = parse_ttl("@prefix ex: <http://example.org/> .\nex:a a ex:Thing .").expect("parse");
        assert_eq!(t[0].p, RDF_TYPE);
    }

    fn expect_malformed(src: &str, needle: &str) -> (usize, usize) {
        match parse_ttl(src) {
            Err(Refusal::GraphMalformed {
                line,
                column,
                detail,
            }) => {
                assert!(
                    detail.contains(needle),
                    "detail '{detail}' missing '{needle}'"
                );
                (line, column)
            }
            other => panic!("expected GraphMalformed({needle}), got {other:?}"),
        }
    }

    #[test]
    fn unterminated_string_names_line_and_column() {
        let (line, column) =
            expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p \"oops .", "unterminated");
        assert_eq!(line, 2);
        assert_eq!(column, 11);
    }

    #[test]
    fn missing_dot_refused() {
        expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p ex:b", "unexpected end");
    }

    #[test]
    fn blank_node_refused() {
        expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p [] .", "blank node");
        expect_malformed("@prefix ex: <http://e/> .\n_:b ex:p ex:c .", "blank node");
    }

    #[test]
    fn refused_constructs_each_named() {
        expect_malformed("@base <http://e/> .", "@base");
        expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p (1 2) .", "collection");
        expect_malformed(
            "@prefix ex: <http://e/> .\nex:a ex:p \"x\"@en .",
            "language tag",
        );
        expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p 1.5 .", "decimal");
        expect_malformed("@prefix ex: <http://e/> .\nex:a ex:p true .", "boolean");
        expect_malformed(
            "@prefix ex: <http://e/> .\nex:a ex:p \"x\"^^ex:t .",
            "datatype",
        );
    }

    #[test]
    fn undeclared_prefix_refused() {
        expect_malformed("nope:a nope:p nope:b .", "undeclared prefix");
    }

    fn expect_cap(src: &str, what: &str) {
        match parse_ttl(src) {
            Err(Refusal::GraphCapExceeded {
                what: w,
                cap,
                actual,
            }) => {
                assert_eq!(w, what);
                assert!(actual > cap, "actual {actual} must exceed cap {cap}");
            }
            other => panic!("expected GraphCapExceeded({what}), got {other:?}"),
        }
    }

    #[test]
    fn ttl_bytes_cap_fires_without_truncation() {
        let src = format!(
            "@prefix ex: <http://e/> .\n# {}\nex:a ex:p ex:b .",
            "x".repeat(MAX_TTL_BYTES)
        );
        expect_cap(&src, "ttl_bytes");
    }

    #[test]
    fn triples_cap_fires() {
        // One statement with 4097 integer objects: fits the byte cap so the
        // triple cap (not the byte cap) is what fires.
        let objects: Vec<String> = (0..=MAX_TRIPLES).map(|i| i.to_string()).collect();
        let small = format!("<u:s> <u:p> {} .", objects.join(","));
        assert!(small.len() <= MAX_TTL_BYTES, "fixture must fit byte cap");
        expect_cap(&small, "triples");
    }

    #[test]
    fn iri_len_cap_fires() {
        let long = "x".repeat(MAX_IRI_LEN + 1);
        expect_cap(&format!("<u:{long}> <u:p> <u:o> ."), "iri_len");
    }

    #[test]
    fn expanded_qname_iri_len_cap_fires() {
        let base = "u:".to_string() + &"b".repeat(200);
        let local = "l".repeat(100);
        expect_cap(
            &format!("@prefix e: <{base}> .\ne:{local} e:p e:o ."),
            "iri_len",
        );
    }

    #[test]
    fn lit_len_cap_fires() {
        let long = "x".repeat(MAX_LIT_LEN + 1);
        expect_cap(&format!("<u:s> <u:p> \"{long}\" ."), "lit_len");
    }

    #[test]
    fn prefix_cap_fires() {
        let mut src = String::new();
        for i in 0..=MAX_PREFIXES {
            src.push_str(&format!("@prefix p{i}: <u:{i}> .\n"));
        }
        expect_cap(&src, "prefixes");
    }

    #[test]
    fn parser_never_panics_on_hostile_inputs() {
        // Empty document is valid (doc := (prefix | stmt)*): zero triples.
        assert_eq!(parse_ttl("").expect("empty doc"), Vec::new());
        assert_eq!(parse_ttl("# only a comment").expect("comment doc"), Vec::new());
        let hostile: &[&str] = &[
            ".",
            ";;;;",
            ",",
            "\u{0}",
            "<u:s> <u:p> \"\u{1}\" .",
            "<u:s> <u:p> 9223372036854775808 .", // i64::MAX + 1
            "<u:s> <u:p> -9223372036854775809 .",
            "<u:s> <u:p> - .",
            "@prefix",
            "@prefix e:",
            "@prefix e: <u:>",
            "<u:s> a ; ; ; .",
            &format!("<u:s> {} <u:o> .", "<u:p> 1 ; ".repeat(500)),
            "<u:s> <u:p> <u:o",
            "\"dangling",
            "^^",
            "e:x",
            "<u:s> <u:p> @en .",
        ];
        for src in hostile {
            let r = parse_ttl(src);
            assert!(r.is_err(), "hostile input must refuse: {src:?}");
        }
        // i64::MIN and i64::MAX are accepted exactly
        let ok = parse_ttl("<u:s> <u:p> -9223372036854775808 , 9223372036854775807 .")
            .expect("i64 bounds parse");
        assert_eq!(ok[0].o, Object::Int(i64::MIN));
        assert_eq!(ok[1].o, Object::Int(i64::MAX));
    }

    #[test]
    fn canonical_string_escaping_is_exact() {
        let t = parse_ttl(r#"<u:s> <u:p> "a\"b\\c\nd\te" ."#).expect("parse");
        let canon = canonical_form(&t);
        assert_eq!(canon, "<u:s> <u:p> \"a\\\"b\\\\c\\nd\\te\" .\n");
    }

    // -----------------------------------------------------------------------
    // vocab / ir / lower
    // -----------------------------------------------------------------------

    const DEMO_TTL: &str = include_str!("../ontology/workflow_demo.ttl");

    fn demo_ir() -> WorkflowIr {
        extract_ir(&parse_ttl(DEMO_TTL).expect("demo parses")).expect("demo extracts")
    }

    #[test]
    fn demo_extraction_round_trip() {
        let ir = demo_ir();
        assert_eq!(ir.budget, 3);
        assert_eq!(
            ir.init,
            vec![IrAtom { predicate: "raw".into(), args: vec!["doc".into()] }]
        );
        assert_eq!(
            ir.goal,
            vec![IrAtom { predicate: "receipted".into(), args: vec!["doc".into()] }]
        );
        let names: Vec<&str> = ir.capabilities.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["gather", "receipt", "verify"], "sorted by name");
        assert!(ir.capabilities.iter().all(|c| c.params == 1 && c.cost == 1));
        assert_eq!(
            ir.constraints,
            vec![IrConstraint {
                kind: "before".into(),
                a: Some("gather".into()),
                b: Some("receipt".into()),
                k: None,
            }]
        );
        // Serde round-trip: same IR, same hash.
        let json = serde_json::to_string(&ir).expect("serialize");
        let back: WorkflowIr = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ir, back);
        assert_eq!(ir_hash(&ir).expect("hash"), ir_hash(&back).expect("hash"));
    }

    #[test]
    fn ir_hash_is_surface_order_invariant() {
        // Same graph with statement blocks reversed: same triples, same IR hash.
        let mut blocks: Vec<&str> = DEMO_TTL.split("\n\n").collect();
        assert!(blocks.len() > 2, "demo has multiple statement blocks");
        let header = blocks.remove(0); // comments + @prefix declarations
        blocks.reverse();
        let reordered = format!("{header}\n\n{}", blocks.join("\n\n"));
        let a = demo_ir();
        let b = extract_ir(&parse_ttl(&reordered).expect("parse")).expect("extract");
        assert_eq!(a, b);
        assert_eq!(ir_hash(&a).expect("hash"), ir_hash(&b).expect("hash"));
    }

    #[test]
    fn demo_lowers_and_solves_to_golden_order() {
        use crate::{Solver, Solver8};
        let ir = demo_ir();
        let (_program, problem) = lower(&ir).expect("lower");
        let plan = Solver8.solve(&problem).expect("solvable");
        let order: Vec<&str> = plan.steps.iter().map(|s| s.capability.as_str()).collect();
        assert_eq!(order, ["gather", "verify", "receipt"]);
        assert_eq!(plan.cost, 3);
    }

    fn wf_doc(body: &str) -> String {
        format!(
            "@prefix wf: <{WF_NS}> .\n@prefix ex: <http://example.org/> .\n{body}"
        )
    }

    fn expect_ill(body: &str, needle: &str) {
        match parse_ttl(&wf_doc(body)).and_then(|t| extract_ir(&t)) {
            Err(Refusal::WorkflowIllFormed { detail, .. }) => assert!(
                detail.contains(needle),
                "detail '{detail}' missing '{needle}'"
            ),
            other => panic!("expected WorkflowIllFormed({needle}), got {other:?}"),
        }
    }

    const MIN_WF: &str = "ex:w a wf:Workflow ; wf:budget 1 ; wf:goal ex:g .\n\
        ex:g a wf:Atom ; wf:predicate \"done\" .\n";

    #[test]
    fn unknown_wf_predicate_names_culprit() {
        let src = wf_doc(&format!("{MIN_WF}ex:w wf:frobnicate 1 ."));
        match parse_ttl(&src).and_then(|t| extract_ir(&t)) {
            Err(Refusal::UnknownPredicate { predicate, subject }) => {
                assert_eq!(predicate, format!("{WF_NS}frobnicate"));
                assert_eq!(subject, "http://example.org/w");
            }
            other => panic!("expected UnknownPredicate, got {other:?}"),
        }
    }

    #[test]
    fn asserted_spec_hash_is_refused_by_name() {
        // The bitactor anti-pattern: the document asserting the hash the
        // receipt must compute. Refused as an unknown predicate.
        let src = wf_doc(&format!("{MIN_WF}ex:w wf:specHash \"b3:deadbeef\" ."));
        match parse_ttl(&src).and_then(|t| extract_ir(&t)) {
            Err(Refusal::UnknownPredicate { predicate, .. }) => {
                assert_eq!(predicate, format!("{WF_NS}specHash"));
            }
            other => panic!("expected UnknownPredicate, got {other:?}"),
        }
    }

    #[test]
    fn foreign_namespace_triples_are_ignored_semantically_but_hashed() {
        let plain = wf_doc(MIN_WF);
        let annotated = wf_doc(&format!(
            "{MIN_WF}ex:w <http://purl.org/dc/terms/creator> \"sean\" ."
        ));
        let a = parse_ttl(&plain).expect("parse");
        let b = parse_ttl(&annotated).expect("parse");
        assert_eq!(
            extract_ir(&a).expect("ir"),
            extract_ir(&b).expect("ir"),
            "foreign triple must not change the IR"
        );
        assert_ne!(graph_hash(&a), graph_hash(&b), "but it is still hashed");
    }

    #[test]
    fn budget_nine_is_a_typed_budget_refusal() {
        let src = wf_doc(
            "ex:w a wf:Workflow ; wf:budget 9 ; wf:goal ex:g .\n\
             ex:g a wf:Atom ; wf:predicate \"done\" .\n",
        );
        match parse_ttl(&src).and_then(|t| extract_ir(&t)) {
            Err(Refusal::BudgetExceeded { what, budget, spent, .. }) => {
                assert_eq!(what, "wf:budget");
                assert_eq!(budget, 8);
                assert_eq!(spent, 9);
            }
            other => panic!("expected BudgetExceeded, got {other:?}"),
        }
    }

    #[test]
    fn workflow_shape_refusals_each_named() {
        // zero workflows
        expect_ill("ex:g a wf:Atom ; wf:predicate \"p\" .", "no 'a wf:Workflow'");
        // two workflows
        expect_ill(
            &format!("{MIN_WF}ex:w2 a wf:Workflow ; wf:budget 1 ; wf:goal ex:g ."),
            "2 wf:Workflow nodes",
        );
        // budget 0
        expect_ill(
            "ex:w a wf:Workflow ; wf:budget 0 ; wf:goal ex:g .\n\
             ex:g a wf:Atom ; wf:predicate \"p\" .",
            "out of range 1..=8",
        );
        // missing budget
        expect_ill(
            "ex:w a wf:Workflow ; wf:goal ex:g .\nex:g a wf:Atom ; wf:predicate \"p\" .",
            "missing wf:budget",
        );
        // missing goal
        expect_ill("ex:w a wf:Workflow ; wf:budget 1 .", "missing wf:goal");
        // goal referencing a node not typed wf:Atom
        expect_ill(
            "ex:w a wf:Workflow ; wf:budget 1 ; wf:goal ex:nope .",
            "not declared 'a wf:Atom'",
        );
        // unknown wf: class
        expect_ill(&format!("{MIN_WF}ex:x a wf:Gizmo ."), "unknown wf: class");
        // atom missing wf:predicate
        expect_ill(
            "ex:w a wf:Workflow ; wf:budget 1 ; wf:goal ex:g .\nex:g a wf:Atom .",
            "missing wf:predicate",
        );
    }

    #[test]
    fn arg_gap_is_refused() {
        expect_ill(
            "ex:w a wf:Workflow ; wf:budget 1 ; wf:goal ex:g .\n\
             ex:g a wf:Atom ; wf:predicate \"p\" ; wf:arg0 \"x\" ; wf:arg2 \"y\" .",
            "argument gap",
        );
    }

    #[test]
    fn var_in_init_is_refused() {
        expect_ill(
            "ex:w a wf:Workflow ; wf:budget 1 ; wf:init ex:i ; wf:goal ex:g .\n\
             ex:i a wf:Atom ; wf:predicate \"raw\" ; wf:arg0 \"?0\" .\n\
             ex:g a wf:Atom ; wf:predicate \"done\" .",
            "init must be ground",
        );
    }

    #[test]
    fn duplicate_capability_name_is_refused() {
        expect_ill(
            &format!(
                "{MIN_WF}\
                 ex:c1 a wf:Capability ; wf:name \"dup\" ; wf:params 0 ; wf:cost 1 .\n\
                 ex:c2 a wf:Capability ; wf:name \"dup\" ; wf:params 0 ; wf:cost 1 ."
            ),
            "duplicate capability name",
        );
    }

    #[test]
    fn capability_field_bounds_are_refused() {
        expect_ill(
            &format!("{MIN_WF}ex:c a wf:Capability ; wf:name \"c\" ; wf:params 9 ; wf:cost 1 ."),
            "wf:params 9 out of range",
        );
        expect_ill(
            &format!("{MIN_WF}ex:c a wf:Capability ; wf:name \"c\" ; wf:params 1 ; wf:cost -1 ."),
            "wf:cost -1 out of range",
        );
        expect_ill(
            &format!("{MIN_WF}ex:c a wf:Capability ; wf:params 1 ; wf:cost 1 ."),
            "missing wf:name",
        );
        expect_ill(
            &format!("{MIN_WF}ex:c a wf:Capability ; wf:name \"c\" ; wf:name \"d\" ; wf:params 1 ; wf:cost 1 ."),
            "multiple wf:name",
        );
    }

    const ONE_CAP: &str = "ex:c a wf:Capability ; wf:name \"c\" ; wf:params 0 ; wf:cost 1 .\n";

    #[test]
    fn constraint_kind_totality_all_eight_map() {
        let cases: &[(&str, Constraint)] = &[
            (
                "wf:kind \"before\" ; wf:a \"c\" ; wf:b \"c\"",
                Constraint::Before { a: "c".into(), b: "c".into() },
            ),
            (
                "wf:kind \"after\" ; wf:a \"c\" ; wf:b \"c\"",
                Constraint::After { a: "c".into(), b: "c".into() },
            ),
            (
                "wf:kind \"excludes\" ; wf:a \"c\" ; wf:b \"c\"",
                Constraint::Excludes { a: "c".into(), b: "c".into() },
            ),
            (
                "wf:kind \"requires\" ; wf:a \"c\" ; wf:b \"c\"",
                Constraint::Requires { a: "c".into(), b: "c".into() },
            ),
            (
                "wf:kind \"not-later\" ; wf:a \"c\" ; wf:k 2",
                Constraint::NotLater { a: "c".into(), k: 2 },
            ),
            (
                "wf:kind \"not-earlier\" ; wf:a \"c\" ; wf:k 2",
                Constraint::NotEarlier { a: "c".into(), k: 2 },
            ),
            (
                "wf:kind \"at-most\" ; wf:a \"c\" ; wf:k 2",
                Constraint::AtMost { a: "c".into(), n: 2 },
            ),
            ("wf:kind \"budget\" ; wf:k 7", Constraint::Budget { max: 7 }),
        ];
        for (body, expected) in cases {
            let src = wf_doc(&format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; {body} ."));
            let ir = extract_ir(&parse_ttl(&src).expect("parse")).expect("extract");
            assert_eq!(ir.constraints.len(), 1);
            assert_eq!(&lower_constraint(&ir.constraints[0]), expected);
        }
    }

    #[test]
    fn constraint_shape_refusals_each_named() {
        // unknown kind
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"eventually\" ; wf:a \"c\" ."),
            "unknown constraint kind",
        );
        // missing required field
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"before\" ; wf:a \"c\" ."),
            "requires wf:b",
        );
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"at-most\" ; wf:a \"c\" ."),
            "requires wf:k",
        );
        // k over the u8 bound for step-indexed kinds
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"not-later\" ; wf:a \"c\" ; wf:k 256 ."),
            "wf:k <= 255",
        );
        // negative k
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"budget\" ; wf:k -1 ."),
            "out of range 0..=u32::MAX",
        );
        // dangling capability name
        expect_ill(
            &format!("{MIN_WF}{ONE_CAP}ex:k a wf:Constraint ; wf:kind \"before\" ; wf:a \"c\" ; wf:b \"ghost\" ."),
            "undeclared capability 'ghost'",
        );
    }

    // -----------------------------------------------------------------------
    // run
    // -----------------------------------------------------------------------

    #[test]
    fn receipt_chain_folds_graph_hash_first_then_each_derived_stage() {
        let receipt = execute_workflow(DEMO_TTL).expect("demo executes");
        // Recompute the chain by hand from the receipt's own stage fields.
        let mut chain = genesis_seed(WORKFLOW_CHAIN_DOMAIN);
        for stage in [
            &receipt.graph_hash, // 1st: the law
            &receipt.ir_hash,
            &receipt.plan_hash,
            &receipt.topology_hash,
            &receipt.geometry_hash,
            &receipt.exec_hash,
        ] {
            chain = fold_event(&chain, stage.as_bytes());
        }
        assert_eq!(chain, receipt.chain, "chain fold order: graph first");
        // ttl_hash is a field only — the chain never folds it.
        assert_ne!(receipt.ttl_hash, receipt.graph_hash);
        // Embedded artifacts agree with the top-level stage hashes.
        assert_eq!(receipt.plan_hash, receipt.plan.receipt.plan_hash);
        assert_eq!(receipt.topology_hash, receipt.supervised.topology_hash);
        assert_eq!(receipt.geometry_hash, receipt.supervised.geometry_hash);
        assert_eq!(receipt.exec_hash, exec_hash_of(&receipt.supervised).expect("hash"));
        assert_eq!(receipt.supervised.outcome, crate::dag::RunOutcome::Completed);
        let order: Vec<&str> =
            receipt.plan.steps.iter().map(|s| s.capability.as_str()).collect();
        assert_eq!(order, ["gather", "verify", "receipt"]);
    }
}
