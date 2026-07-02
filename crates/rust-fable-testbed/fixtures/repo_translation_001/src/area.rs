//! Already-correct sibling module: computes area for a [`crate::Shape`], including
//! `label` in its output. Given to the model as read-only context (correct, not the
//! task's target file) so it can see the pattern `describe.rs` needs to follow.

use crate::Shape;

/// Return a human-readable area summary for `shape`, including its label.
#[must_use]
pub fn describe_area(shape: &Shape) -> String {
    format!("{} area: {:.2}", shape.label, shape.width * shape.height)
}

#[cfg(test)]
mod tests {
    use super::describe_area;
    use crate::Shape;

    #[test]
    fn includes_label_and_computed_area() {
        let shape = Shape::new(3.0, 4.0, "tile");
        assert_eq!(describe_area(&shape), "tile area: 12.00");
    }
}
