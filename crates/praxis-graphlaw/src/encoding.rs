use crate::{BlankNodeImpl, LiteralImpl, Term, TermImpl};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static GLOBAL_ENCODER: Lazy<Mutex<InternalEncoder>> =
    Lazy::new(|| Mutex::new(InternalEncoder::new()));

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum EncodedValue {
    Iri(String),
    LiteralLexical(String),
    BlankNodeLabel(String),
    Literal {
        value: usize,            // Index of the lexical value string
        datatype: Option<usize>, // Index of the datatype IRI
        lang: Option<usize>,     // Index of the language tag
    },
    Variable(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InternalEncoder {
    encoded: HashMap<EncodedValue, usize>,
    decoded: HashMap<usize, EncodedValue>,
    counter: usize,
}

impl InternalEncoder {
    pub fn new() -> InternalEncoder {
        InternalEncoder {
            encoded: HashMap::new(),
            decoded: HashMap::new(),
            counter: 0,
        }
    }

    pub fn add_iri(&mut self, iri: String) -> usize {
        let val = EncodedValue::Iri(iri);
        self.intern(val)
    }

    pub fn add_literal_lexical(&mut self, lexical: String) -> usize {
        let val = EncodedValue::LiteralLexical(lexical);
        self.intern(val)
    }

    pub fn add_blank_node_label(&mut self, label: String) -> usize {
        let val = EncodedValue::BlankNodeLabel(label);
        self.intern(val)
    }

    pub fn add_variable(&mut self, var: String) -> usize {
        let val = EncodedValue::Variable(var);
        self.intern(val)
    }

    pub fn add_literal(
        &mut self,
        value: String,
        datatype: Option<String>,
        lang: Option<String>,
    ) -> usize {
        let value_id = self.add_literal_lexical(value);
        let datatype_id = datatype.map(|dt| self.add_iri(dt));
        let lang_id = lang.map(|l| self.add_literal_lexical(l));
        let val = EncodedValue::Literal {
            value: value_id,
            datatype: datatype_id,
            lang: lang_id,
        };
        self.intern(val)
    }

    fn intern(&mut self, val: EncodedValue) -> usize {
        if let Some(id) = self.encoded.get(&val) {
            *id
        } else {
            let id = self.counter;
            self.encoded.insert(val.clone(), id);
            self.decoded.insert(id, val);
            self.counter += 1;
            id
        }
    }

    pub fn add(&mut self, s: String) -> usize {
        if s.starts_with('?') {
            self.add_variable(s)
        } else if s.starts_with("_:") {
            self.add_blank_node_label(s[2..].to_string())
        } else if s.starts_with('"') {
            let last_quote = s.rfind('"');
            if let Some(end_lex) = last_quote {
                if end_lex > 0 {
                    let lexical = s[1..end_lex].to_string();
                    let suffix = &s[end_lex + 1..];
                    let mut datatype = None;
                    let mut lang = None;
                    if suffix.starts_with('@') {
                        lang = Some(suffix[1..].to_string());
                    } else if suffix.starts_with("^^") {
                        datatype = Some(suffix[2..].to_string());
                    }
                    self.add_literal(lexical, datatype, lang)
                } else {
                    self.add_iri(s)
                }
            } else {
                self.add_iri(s)
            }
        } else {
            self.add_iri(s)
        }
    }

    pub fn get(&self, s: &str) -> Option<usize> {
        if s.starts_with('?') {
            self.encoded
                .get(&EncodedValue::Variable(s.to_string()))
                .copied()
        } else {
            if let Some(id) = self.encoded.get(&EncodedValue::Iri(s.to_string())).copied() {
                Some(id)
            } else if let Some(id) = self.encoded.get(&EncodedValue::Variable(format!("?{}", s))).copied() {
                Some(id)
            } else if s.starts_with("_:") {
                self.encoded
                    .get(&EncodedValue::BlankNodeLabel(s[2..].to_string()))
                    .copied()
            } else if s.starts_with('"') {
                let last_quote = s.rfind('"');
                if let Some(end_lex) = last_quote {
                    if end_lex > 0 {
                        let lexical = s[1..end_lex].to_string();
                        let suffix = &s[end_lex + 1..];
                        let mut datatype = None;
                        let mut lang = None;
                        if suffix.starts_with('@') {
                            lang = Some(suffix[1..].to_string());
                        } else if suffix.starts_with("^^") {
                            datatype = Some(suffix[2..].to_string());
                        }
                        let val_id = self.encoded.get(&EncodedValue::LiteralLexical(lexical))?;
                        let datatype_id = if let Some(dt) = datatype {
                            Some(*self.encoded.get(&EncodedValue::Iri(dt))?)
                        } else {
                            None
                        };
                        let lang_id = if let Some(l) = lang {
                            Some(*self.encoded.get(&EncodedValue::LiteralLexical(l))?)
                        } else {
                            None
                        };
                        let lit_val = EncodedValue::Literal {
                            value: *val_id,
                            datatype: datatype_id,
                            lang: lang_id,
                        };
                        self.encoded.get(&lit_val).copied()
                    } else {
                        self.encoded.get(&EncodedValue::Iri(s.to_string())).copied()
                    }
                } else {
                    self.encoded.get(&EncodedValue::Iri(s.to_string())).copied()
                }
            } else {
                self.encoded.get(&EncodedValue::Iri(s.to_string())).copied()
            }
        }
    }

    pub fn decode(&self, encoded: &usize) -> Option<String> {
        match self.decoded.get(encoded)? {
            EncodedValue::Iri(s) => Some(s.clone()),
            EncodedValue::LiteralLexical(s) => Some(s.clone()),
            EncodedValue::BlankNodeLabel(label) => Some(format!("_:{}", label)),
            EncodedValue::Variable(s) => Some(s.clone()),
            EncodedValue::Literal {
                value,
                datatype,
                lang,
            } => {
                let val_str = match self.decoded.get(value)? {
                    EncodedValue::LiteralLexical(s) => s.clone(),
                    _ => return None,
                };
                let mut res = format!("\"{}\"", val_str);
                if let Some(lang_id) = lang {
                    if let EncodedValue::LiteralLexical(lang_str) = self.decoded.get(lang_id)? {
                        res.push_str(&format!("@{}", lang_str));
                    }
                } else if let Some(dt_id) = datatype {
                    if let EncodedValue::Iri(dt_str) = self.decoded.get(dt_id)? {
                        res.push_str(&format!("^^{}", dt_str));
                    }
                }
                Some(res)
            }
        }
    }

    pub fn decode_to_term(&self, id: usize) -> Option<Term> {
        match self.decoded.get(&id)? {
            EncodedValue::Iri(_) => Some(Term::Iri(TermImpl { iri: id })),
            EncodedValue::BlankNodeLabel(_) => Some(Term::BlankNode(BlankNodeImpl { id })),
            EncodedValue::Literal {
                value,
                datatype,
                lang,
            } => Some(Term::Literal(LiteralImpl {
                id,
                value: *value,
                datatype: *datatype,
                lang: *lang,
            })),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Encoder {}

impl Encoder {
    pub fn add(uri: String) -> usize {
        let mut encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.add(uri)
    }

    pub fn get(uri: &str) -> Option<usize> {
        let encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.get(uri)
    }

    pub fn decode(encoded: &usize) -> Option<String> {
        let encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.decode(encoded)
    }

    pub fn add_iri(iri: String) -> usize {
        let mut encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.add_iri(iri)
    }

    pub fn add_blank_node(label: String) -> usize {
        let mut encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.add_blank_node_label(label)
    }

    pub fn add_literal(value: String, datatype: Option<String>, lang: Option<String>) -> usize {
        let mut encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.add_literal(value, datatype, lang)
    }

    pub fn decode_to_term(id: usize) -> Option<Term> {
        let encoder = GLOBAL_ENCODER.lock().unwrap();
        encoder.decode_to_term(id)
    }
}

#[test]
fn test_encoding() {
    let mut encoder = InternalEncoder::new();
    let _encoded1 = encoder.add("http://test/1".to_string());
    let encoded2 = encoder.add("http://test/2".to_string());
    let encoded3 = encoder.add("http://test/3".to_string());
    let decoded2 = encoder.decode(&encoded2);
    let decoded2_again = encoder.decode(&encoded2);
    assert_eq!("http://test/2", decoded2.unwrap());
    assert_eq!("http://test/2", decoded2_again.unwrap());
    assert_eq!(2, encoded3);
}

#[test]
fn test_encoder_literal_vs_iri_distinct() {
    // Add same lexical value string as an IRI and as a literal, verify IDs are distinct.
    let iri_str = "http://example.com/foo".to_string();
    let lit_str = "\"http://example.com/foo\"".to_string();

    let iri_id = Encoder::add(iri_str.clone());
    let lit_id = Encoder::add(lit_str.clone());

    assert_ne!(iri_id, lit_id, "IRI and Literal must have distinct IDs");

    let decoded_iri = Encoder::decode(&iri_id).unwrap();
    let decoded_lit = Encoder::decode(&lit_id).unwrap();

    assert_eq!(decoded_iri, "http://example.com/foo");
    assert_eq!(decoded_lit, "\"http://example.com/foo\"");
}

#[test]
fn test_literal_datatype_and_langtag_preserved() {
    let lit_with_dt = "\"10\"^^<http://www.w3.org/2001/XMLSchema#integer>".to_string();
    let lit_with_lang = "\"hello\"@en".to_string();

    let dt_id = Encoder::add(lit_with_dt.clone());
    let lang_id = Encoder::add(lit_with_lang.clone());

    assert_eq!(Encoder::decode(&dt_id).unwrap(), lit_with_dt);
    assert_eq!(Encoder::decode(&lang_id).unwrap(), lit_with_lang);

    // Verify decode_to_term works
    let term_dt = Encoder::decode_to_term(dt_id).unwrap();
    if let Term::Literal(lit) = term_dt {
        assert_eq!(Encoder::decode(&lit.id).unwrap(), lit_with_dt);
        assert_eq!(Encoder::decode(&lit.value).unwrap(), "10");
        assert_eq!(
            Encoder::decode(&lit.datatype.unwrap()).unwrap(),
            "<http://www.w3.org/2001/XMLSchema#integer>"
        );
        assert!(lit.lang.is_none());
    } else {
        panic!("Expected Term::Literal");
    }

    let term_lang = Encoder::decode_to_term(lang_id).unwrap();
    if let Term::Literal(lit) = term_lang {
        assert_eq!(Encoder::decode(&lit.id).unwrap(), lit_with_lang);
        assert_eq!(Encoder::decode(&lit.value).unwrap(), "hello");
        assert!(lit.datatype.is_none());
        assert_eq!(Encoder::decode(&lit.lang.unwrap()).unwrap(), "en");
    } else {
        panic!("Expected Term::Literal");
    }
}
