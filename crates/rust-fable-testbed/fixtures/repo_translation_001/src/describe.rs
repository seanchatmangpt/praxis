//! Target module for `repo_translation_001`: `describe_shape` still summarizes a
//! [`crate::Shape`] as if it only had `width`/`height`, dropping the newer `label`
//! field that [`crate::area::describe_area`] (a sibling module, already fixed) already
//! accounts for. Compiles fine (the field is public and simply unused here), but fails
//! `describe_shape_includes_the_label` below. The model must update this file only.

use crate::Shape;

/// Return a human-readable summary of `shape`.
///
/// # Bug
///
/// This omits `shape.label` entirely, even though `Shape` has carried a `label` field
/// (used correctly by the sibling `area::describe_area`) since it was added. Update
/// this function so its output includes the label, following the same
/// `"{label} ..."` prefix convention `area::describe_area` already uses.
#[must_use]
pub fn describe_shape(shape: &Shape) -> String {
    format!("{}x{} shape", shape.width, shape.height)
}

#[cfg(test)]
mod tests {
    use super::describe_shape;
    use crate::Shape;

    #[test]
    fn describe_shape_includes_dimensions() {
        let shape = Shape::new(3.0, 4.0, "tile");
        let out = describe_shape(&shape);
        assert!(out.contains('3'));
        assert!(out.contains('4'));
    }

    /// This is the bug: with the `label` field now present on `Shape` (and already
    /// used correctly by `area::describe_area`), `describe_shape` must include it too.
    #[test]
    fn describe_shape_includes_the_label() {
        let shape = Shape::new(3.0, 4.0, "tile");
        let out = describe_shape(&shape);
        assert!(out.contains("tile"), "expected output to include the shape's label, got: {out}");
    }
}
