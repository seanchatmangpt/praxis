//! Fixture module with three raw-pointer blocks (see the paired task spec for the
//! full audit brief).
//!
//! Two of the three blocks are sound and already documented with a `SAFETY:`
//! justification comment (the convention `safety_audit` in `pipeline.rs` looks for).
//! The third, [`last_or_zero`], is genuinely unsound: it reads one element past the
//! end of the slice instead of the actual last element, which is both an
//! out-of-bounds heap read (UB, Miri-detectable) and simply the wrong value. The
//! model under test must fix `last_or_zero` without touching the two sound
//! functions (or, if it prefers, replace the risky block with a safe alternative
//! where one exists — `last_or_zero` has an entirely safe implementation).

/// Intended to return the last element of `arr` (or `0` for an empty slice), but
/// instead reads one element *past* the end of the slice.
///
/// # Bug
///
/// `ptr.add(arr.len())` points one element past the end of `arr`'s buffer.
/// Dereferencing it is out-of-bounds (a heap-buffer-overflow read) — undefined
/// behavior regardless of what value happens to land there, and (setting UB aside)
/// it is simply not `arr[arr.len() - 1]`. Detectable both by the failing test below
/// (the returned value is not the real last element) and, definitively, by
/// running this fixture's tests under Miri (see the paired task spec's fixture
/// path), which reports the out-of-bounds pointer access directly.
///
/// Fix: read `arr[arr.len() - 1]` (or, since no raw-pointer trick is actually
/// needed here, drop the pointer arithmetic entirely and use safe slice indexing /
/// `arr.last()`).
#[must_use]
pub fn last_or_zero(arr: &[i32]) -> i32 {
    if arr.is_empty() {
        return 0;
    }
    let ptr = arr.as_ptr();
    // BUG: should be `ptr.add(arr.len() - 1)` — this is one past the end.
    unsafe { *ptr.add(arr.len()) }
}

/// Sound: sum the elements of `arr` at each index in `indices`, using
/// `get_unchecked` for the actual reads only after validating that every index is
/// in-bounds.
#[must_use]
pub fn sum_at_indices(arr: &[i32], indices: &[usize]) -> i32 {
    for &i in indices {
        assert!(i < arr.len(), "index {i} out of bounds for slice of len {}", arr.len());
    }
    let mut total = 0i32;
    for &i in indices {
        // SAFETY: every index in `indices` was checked to be `< arr.len()` in the
        // loop above, so `arr.get_unchecked(i)` is always a valid, in-bounds read.
        total += unsafe { *arr.get_unchecked(i) };
    }
    total
}

/// Sound: swap the first and last elements of `arr` via raw-pointer writes, after
/// checking `arr.len() >= 2`.
pub fn swap_first_last(arr: &mut [i32]) {
    let len = arr.len();
    if len < 2 {
        return;
    }
    let ptr = arr.as_mut_ptr();
    // SAFETY: `len >= 2` checked above, so `ptr` and `ptr.add(len - 1)` are both
    // valid, in-bounds, non-overlapping pointers into `arr`, satisfying `ptr::swap`.
    unsafe {
        std::ptr::swap(ptr, ptr.add(len - 1));
    }
}

#[cfg(test)]
mod tests {
    use super::{last_or_zero, sum_at_indices, swap_first_last};

    /// This is the bug: `last_or_zero` must return the real last element of `arr`,
    /// but the current implementation reads one element past the end of the slice
    /// instead, so this assertion fails against the unfixed fixture. Also
    /// deterministically reported as an out-of-bounds heap read by
    /// running this fixture's tests under Miri.
    #[test]
    fn last_or_zero_returns_the_actual_last_element() {
        let arr = [10, 20, 30];
        assert_eq!(last_or_zero(&arr), 30);
    }

    #[test]
    fn last_or_zero_returns_zero_for_empty_slice() {
        let arr: [i32; 0] = [];
        assert_eq!(last_or_zero(&arr), 0);
    }

    #[test]
    fn sum_at_indices_sums_selected_elements() {
        let arr = [10, 20, 30, 40];
        assert_eq!(sum_at_indices(&arr, &[0, 2, 3]), 80);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn sum_at_indices_panics_on_out_of_bounds_index() {
        let arr = [10, 20, 30];
        let _ = sum_at_indices(&arr, &[5]);
    }

    #[test]
    fn swap_first_last_swaps_endpoints() {
        let mut arr = [1, 2, 3, 4];
        swap_first_last(&mut arr);
        assert_eq!(arr, [4, 2, 3, 1]);
    }

    #[test]
    fn swap_first_last_is_noop_for_short_slices() {
        let mut arr = [1];
        swap_first_last(&mut arr);
        assert_eq!(arr, [1]);

        let mut empty: [i32; 0] = [];
        swap_first_last(&mut empty);
        assert_eq!(empty, []);
    }
}
