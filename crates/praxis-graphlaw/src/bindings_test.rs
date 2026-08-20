#![cfg(test)]

use super::*;
use crate::term::Triple;
use crate::tripleindex::TripleIndex;
use crate::{Parser, Syntax};

/// Reachability confirmation for the `Binding::len()` HashMap-iteration-order
/// issue named in `Binding::len()`'s own doc comment
/// (`docs/releases/v26.7.13/RELEASE_CONTROL.md` Sec. 5 row 4): does
/// `Binding::combine()` actually produce a binding whose columns have
/// genuinely different lengths ("ragged") from real,
/// `TripleIndex::query()`-constructed inputs, or was the concern purely
/// theoretical?
///
/// `combine()`'s current implementation computes `binding_size` as the max
/// column length already present in the *receiving* binding, then repeats
/// every value of each *incoming* column that many times -- it does not use
/// the incoming binding's own natural row count. So a single combine() of
/// two bindings whose sizes happen to already match (e.g. 3 and 1, where the
/// lone incoming row gets repeated 3 times) does not by itself produce a
/// ragged result. Raggedness appears on the *next* combine(), once the
/// receiving binding already holds columns of different lengths and a third
/// column is folded in at the (now-larger) `binding_size` rather than at its
/// own natural size.
///
/// This test drives that real three-step sequence through real
/// `TripleIndex::query()` results (three subjects share `p`/`x`, one subject
/// has `q`/`y`, two subjects share `r`/`z`) and asserts the resulting
/// `Binding` genuinely carries differently-sized columns, rather than
/// reasoning about it in a doc comment.
///
/// This does not fix `combine()`'s ragged-column production (a separate,
/// deeper semantic issue in its row-multiplication logic, out of scope
/// here) -- it only confirms the reachability claim `Binding::len()`'s doc
/// comment makes, so that claim rests on an executed test instead of an
/// assertion in prose.
#[test]
fn test_combine_of_real_query_bindings_produces_ragged_columns_reachability_confirmed() {
    let mut index = TripleIndex::new();
    let data = "<http://example.com/a> <http://example.com/p> <http://example.com/x>.\n\
                <http://example.com/b> <http://example.com/p> <http://example.com/x>.\n\
                <http://example.com/c> <http://example.com/p> <http://example.com/x>.\n\
                <http://example.com/only> <http://example.com/q> <http://example.com/y>.\n\
                <http://example.com/d> <http://example.com/r> <http://example.com/z>.\n\
                <http://example.com/e> <http://example.com/r> <http://example.com/z>.";
    let triples =
        Parser::parse_triples(data, Syntax::NQuads).expect("fixture data must parse as N-Quads");
    triples.into_iter().for_each(|t| index.add(t));

    let query = |var: &str, pred: &str, obj: &str| -> Binding {
        let t = Triple::from(
            var.to_string(),
            format!("http://example.com/{pred}"),
            format!("http://example.com/{obj}"),
        );
        index
            .query(&t, None)
            .unwrap_or_else(|| panic!("query {var} {pred} {obj} must match a real fact"))
    };

    // ?s p x -- three subjects match.
    let b_s = query("?s", "p", "x");
    assert_eq!(
        b_s.len(),
        3,
        "sanity: three real rows for ?s before combine()"
    );

    // ?t q y -- exactly one subject matches.
    let b_t = query("?t", "q", "y");
    assert_eq!(b_t.len(), 1, "sanity: one real row for ?t before combine()");

    // ?u r z -- two subjects match.
    let b_u = query("?u", "r", "z");
    assert_eq!(
        b_u.len(),
        2,
        "sanity: two real rows for ?u before combine()"
    );

    let s_var = *b_s.vars().first().expect("?s binding has one variable");
    let t_var = *b_t.vars().first().expect("?t binding has one variable");
    let u_var = *b_u.vars().first().expect("?u binding has one variable");

    let mut combined = b_s.clone();
    combined.combine(b_t.clone()); // binding_size = 3 (from ?s); ?t's single row repeats 3x.
    combined.combine(b_u.clone()); // binding_size = 3 (max of ?s/?t so far); ?u's two rows repeat 3x each -> 6.

    let s_col_len = combined
        .get(s_var)
        .expect("?s column must still be present after combine()")
        .len();
    let t_col_len = combined
        .get(t_var)
        .expect("?t column must be present after combine()")
        .len();
    let u_col_len = combined
        .get(u_var)
        .expect("?u column must be present after combine()")
        .len();

    assert_eq!(
        s_col_len, 3,
        "combine() must not alter the receiving binding's own existing column length"
    );
    assert_eq!(
        t_col_len, 3,
        "?t's single row repeated at the receiving binding's size (3) after the first combine()"
    );
    assert_eq!(
        u_col_len, 6,
        "reachability confirmed: ?u's two rows repeated at the (still 3) receiving \
         binding_size after the second combine() yields 6, genuinely different from \
         the ?s/?t columns' length of 3 -- this is the ragged-column case \
         Binding::len()'s doc comment names as previously unconfirmed"
    );
    assert_ne!(u_col_len, s_col_len, "columns are ragged: 3 vs 6");

    // `len()` must still be deterministic across repeated calls on this
    // exact ragged binding, regardless of which column HashMap iteration
    // would otherwise have visited first.
    let first = combined.len();
    for _ in 0..8 {
        assert_eq!(
            combined.len(),
            first,
            "len() must read the same column deterministically on a ragged binding"
        );
    }
    // The documented fix selects the column for the numerically smallest
    // variable id, so the deterministic result must equal that column's
    // length, not an arbitrary one.
    let smallest_var = *[s_var, t_var, u_var].iter().min().unwrap();
    let expected = combined.get(&smallest_var).unwrap().len();
    assert_eq!(
        first, expected,
        "len() must read the smallest-variable-id column, per its own fix"
    );
}
