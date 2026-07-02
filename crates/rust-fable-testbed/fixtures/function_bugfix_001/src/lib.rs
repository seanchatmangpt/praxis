//! Fixture for `function_bugfix_001`: a `binary_search` implementation with a
//! deliberate off-by-one bug on duplicate keys.
//!
//! The function compiles and correctly handles arrays with unique elements, but
//! returns the *first index probed* rather than the *leftmost matching index* when
//! the target value appears more than once. This is the bug the model under test is
//! expected to fix.

/// Search `arr` (must be sorted ascending) for `target`.
///
/// # Bug
///
/// When `target` appears multiple times in `arr`, this implementation returns
/// whichever matching index binary search happens to probe first, instead of the
/// leftmost (smallest) matching index. Callers that rely on leftmost-index semantics
/// (e.g. to compute the start of a run of duplicates) get a wrong answer.
#[must_use]
pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
    let mut lo = 0usize;
    let mut hi = arr.len();

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if arr[mid] == target {
            // BUG: should keep searching left (`hi = mid`) to find the leftmost
            // occurrence when duplicates are present; instead it returns immediately.
            return Some(mid);
        } else if arr[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::binary_search;

    #[test]
    fn finds_unique_element() {
        let arr = [1, 3, 5, 7, 9, 11];
        assert_eq!(binary_search(&arr, 7), Some(3));
    }

    #[test]
    fn returns_none_when_absent() {
        let arr = [2, 4, 6, 8];
        assert_eq!(binary_search(&arr, 5), None);
    }

    /// This is the bug: with duplicate keys, `binary_search` must return the
    /// *leftmost* matching index, but the current implementation returns whichever
    /// index the probe sequence lands on first.
    #[test]
    fn finds_leftmost_index_with_duplicate_keys() {
        let arr = [1, 2, 3, 3, 3, 3, 3, 8, 9];
        assert_eq!(binary_search(&arr, 3), Some(2));
    }
}
