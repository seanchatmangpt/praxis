// SPARQL CONSTRUCT query parsing and quad serialization (PROJ-408)
// FIX #6: tokenize_triple lone delimiter handling (line 2017-2019)
// FIX #10: expand_semicolon_predicates RDF shorthand expansion

use crate::term::Triple;

pub struct ConstructQuery {
    pub graph: Option<String>,
    pub template_triples: Vec<(String, String, String)>,
    pub where_query: String,
    pub is_delete: bool,
}

pub fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    for line in s.lines() {
        if let Some(idx) = line.find('#') {
            out.push_str(&line[..idx]);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Tokenizes a triple pattern into its components (subject, predicate, object).
/// Handles IRI literals (<...>), quoted literals ("..." or '...'), and bare terms.
/// Recognizes delimiters: '.', ';', '}'
///
/// # FIX #6
/// **Lone delimiter handling** (lines 2016-2020 in original):
/// When a lone delimiter (';', '.', or '}') is encountered that doesn't belong to
/// a token, the character must be consumed via `chars.next()` so the outer loop
/// makes progress. Without this fix, an infinite loop occurs when the peek returns
/// a delimiter but no token is built, leaving the iterator unmoved.
pub fn tokenize_triple(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '<' {
            let mut iri = String::new();
            while let Some(c) = chars.next() {
                iri.push(c);
                if c == '>' {
                    break;
                }
            }
            tokens.push(iri);
        } else if c == '"' || c == '\'' {
            let quote = chars.next().unwrap();
            let mut lit = String::new();
            lit.push(quote);
            while let Some(c) = chars.next() {
                lit.push(c);
                if c == quote {
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_whitespace() || next_c == '.' || next_c == ';' {
                            break;
                        }
                        lit.push(chars.next().unwrap());
                    }
                    break;
                }
            }
            tokens.push(lit);
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '.' || c == ';' || c == '}' {
                    break;
                }
                token.push(chars.next().unwrap());
            }
            if token.is_empty() {
                // Lone delimiter (';', '.', or '}') — consume it so outer loop makes progress.
                chars.next();
                continue;
            }
            tokens.push(token);
        }
    }
    Ok(tokens)
}

/// Expands RDF shorthand: "s p1 o1 ; p2 o2 ; p3 o3" → "s p1 o1 . s p2 o2 . s p3 o3"
/// Semicolons separate predicate-object pairs that share the same subject.
///
/// # FIX #10
/// **Replace ; with .** and reset token count to 1 (subject already output).
/// This transformation allows triple patterns using RDF's semicolon shorthand
/// to be correctly parsed into individual triples by subsequent tokenization.
fn expand_semicolon_predicates(text: &str) -> String {
    let mut result = String::new();
    let mut current_subject: Option<String> = None;
    let mut chars = text.chars().peekable();
    let mut in_iri = false;
    let mut in_literal = false;
    let mut literal_delim = ' ';
    let mut token_buffer = String::new();
    let mut token_count = 0;

    while let Some(&c) = chars.peek() {
        if in_literal {
            token_buffer.push(c);
            chars.next();
            if c == literal_delim {
                // Check for escaped quote
                if chars.peek() != Some(&'\\') {
                    in_literal = false;
                }
            }
        } else if in_iri {
            token_buffer.push(c);
            chars.next();
            if c == '>' {
                in_iri = false;
            }
        } else if c == '<' {
            in_iri = true;
            token_buffer.push(c);
            chars.next();
        } else if c == '"' || c == '\'' {
            in_literal = true;
            literal_delim = c;
            token_buffer.push(c);
            chars.next();
        } else if c.is_whitespace() {
            if !token_buffer.is_empty() {
                token_count += 1;
                if token_count == 1 {
                    current_subject = Some(token_buffer.clone());
                    result.push_str(&token_buffer);
                } else {
                    result.push(' ');
                    result.push_str(&token_buffer);
                }
                token_buffer.clear();
            }
            result.push(c);
            chars.next();
        } else if c == ';' {
            if !token_buffer.is_empty() {
                token_count += 1;
                result.push(' ');
                result.push_str(&token_buffer);
                token_buffer.clear();
            }
            // Replace ; with . and reset token count to 1 (subject already output)
            result.push('.');
            if let Some(ref subj) = current_subject {
                result.push(' ');
                result.push_str(subj);
                token_count = 1;
            }
            chars.next();
        } else {
            token_buffer.push(c);
            chars.next();
        }
    }

    // Output any remaining token
    if !token_buffer.is_empty() {
        if token_count > 0 {
            result.push(' ');
        }
        result.push_str(&token_buffer);
    }

    result
}

pub fn parse_construct(query_str: &str) -> Result<ConstructQuery, String> {
    let clean_query = strip_comments(query_str);
    let const_idx = clean_query
        .to_uppercase()
        .find("CONSTRUCT")
        .ok_or_else(|| "Not a CONSTRUCT query".to_string())?;
    let first_brace = clean_query[const_idx..]
        .find('{')
        .ok_or_else(|| "Missing opening brace for CONSTRUCT template".to_string())?
        + const_idx;

    let mut brace_count = 1;
    let mut template_end = None;
    for (idx, c) in clean_query[first_brace + 1..].char_indices() {
        if c == '{' {
            brace_count += 1;
        } else if c == '}' {
            brace_count -= 1;
            if brace_count == 0 {
                template_end = Some(first_brace + 1 + idx);
                break;
            }
        }
    }
    let template_end =
        template_end.ok_or_else(|| "Unmatched brace in CONSTRUCT template".to_string())?;
    let template_text = clean_query[first_brace + 1..template_end].trim();

    let where_text = clean_query[template_end + 1..].trim();
    if regex::Regex::new(r"(?i)\bSERVICE\b")
        .unwrap()
        .is_match(where_text)
    {
        return Err("unsupported SERVICE clause".to_string());
    }
    let where_pattern = if where_text.to_uppercase().starts_with("WHERE") {
        where_text[5..].trim()
    } else {
        where_text
    };

    let mut graph = None;
    let mut template_triples = Vec::new();
    let is_delete = template_text.is_empty();

    if !is_delete {
        let mut inner_text = template_text;
        if template_text.to_uppercase().starts_with("GRAPH") {
            let first_g_brace = template_text
                .find('{')
                .ok_or_else(|| "Missing brace in GRAPH template".to_string())?;
            let g_part = template_text[5..first_g_brace].trim();
            graph = Some(g_part.to_string());

            let last_g_brace = template_text
                .rfind('}')
                .ok_or_else(|| "Missing closing brace in GRAPH template".to_string())?;
            inner_text = template_text[first_g_brace + 1..last_g_brace].trim();
        }

        // Expand semicolon-joined predicates: "s p1 o1 ; p2 o2 ; p3 o3" → "s p1 o1 . s p2 o2 . s p3 o3"
        let expanded_text = expand_semicolon_predicates(inner_text);

        for triple_part in expanded_text.split('.') {
            let triple_part = triple_part.trim();
            if triple_part.is_empty() {
                continue;
            }
            let tokens = tokenize_triple(triple_part)?;
            if tokens.len() == 3 {
                template_triples.push((tokens[0].clone(), tokens[1].clone(), tokens[2].clone()));
            }
        }
    }

    let select_query = format!("SELECT * WHERE {}", where_pattern);

    Ok(ConstructQuery {
        graph,
        template_triples,
        where_query: select_query,
        is_delete,
    })
}

pub fn get_where_triple_pattern(where_pattern: &str) -> Option<(String, String, String)> {
    let clean = where_pattern.trim();
    let start = clean.find('{')? + 1;
    let end = clean.rfind('}')?;
    let inner = &clean[start..end];
    let tokens = tokenize_triple(inner).ok()?;
    if tokens.len() >= 3 {
        Some((tokens[0].clone(), tokens[1].clone(), tokens[2].clone()))
    } else {
        None
    }
}

pub fn serialize_quad(t: &Triple) -> String {
    let s_str = crate::encoding::Encoder::decode(&t.s.to_encoded()).unwrap_or_default();
    let p_str = crate::encoding::Encoder::decode(&t.p.to_encoded()).unwrap_or_default();
    let o_str = crate::encoding::Encoder::decode(&t.o.to_encoded()).unwrap_or_default();
    let g_str =
        t.g.as_ref()
            .map(|g| crate::encoding::Encoder::decode(&g.to_encoded()).unwrap_or_default());

    let format_val = |s: &str| -> String {
        let s = s.trim();
        if s.starts_with('?') {
            s.to_string()
        } else if s.starts_with('<') && s.ends_with('>') {
            s.to_string()
        } else if s.starts_with('_') && s.contains(':') {
            s.to_string()
        } else if s.starts_with('"') {
            s.to_string()
        } else {
            format!("<{}>", s)
        }
    };

    let s_fmt = format_val(&s_str);
    let p_fmt = format_val(&p_str);
    let o_fmt = format_val(&o_str);
    let g_fmt = g_str
        .map(|g| format!(" {}", format_val(&g)))
        .unwrap_or_default();

    format!("{} {} {}{} .", s_fmt, p_fmt, o_fmt, g_fmt)
}

pub fn escape_literal(s: &str) -> String {
    if !s.starts_with('"') {
        return s.to_string();
    }
    if let Some(end_quote) = s[1..].rfind('"') {
        let end_quote = end_quote + 1;
        let value = &s[1..end_quote];
        let suffix = &s[end_quote + 1..];

        let escaped_value: String = value
            .chars()
            .flat_map(|c| match c {
                '\\' => vec!['\\', '\\'],
                '"' => vec!['\\', '"'],
                '\n' => vec!['\\', 'n'],
                '\r' => vec!['\\', 'r'],
                '\t' => vec!['\\', 't'],
                other => vec![other],
            })
            .collect();

        format!("\"{}\"{}", escaped_value, suffix)
    } else {
        s.to_string()
    }
}

pub fn canonicalize_quads(quads: &[Triple]) -> Vec<String> {
    let mut raw_lines: Vec<String> = quads.iter().map(serialize_quad).collect();
    let mut bnodes = std::collections::BTreeSet::new();
    for line in &raw_lines {
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '_' && chars.peek() == Some(&':') {
                chars.next();
                let mut label = String::from("_:");
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        label.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                bnodes.insert(label);
            }
        }
    }

    if !bnodes.is_empty() {
        let mut bnode_vec: Vec<String> = bnodes.into_iter().collect();
        bnode_vec.sort_by(|a, b| b.len().cmp(&a.len()));

        let c14n_mappings: Vec<(String, String)> = bnode_vec
            .iter()
            .enumerate()
            .map(|(idx, bnode)| (bnode.clone(), format!("_:c14n{}", idx)))
            .collect();

        for line in &mut raw_lines {
            *line = escape_literal(line);
            for (bnode, c14n) in &c14n_mappings {
                let mut new_line = String::new();
                let mut parts = line.split(bnode);
                if let Some(first) = parts.next() {
                    new_line.push_str(first);
                    for part in parts {
                        new_line.push_str(c14n);
                        new_line.push_str(part);
                    }
                }
                *line = new_line;
            }
        }
    } else {
        for line in &mut raw_lines {
            *line = escape_literal(line);
        }
    }

    raw_lines.sort();
    raw_lines
}
