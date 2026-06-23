//! Handle-based object store.
//!
//! Rust objects live here; callers receive opaque hex string handles.
//!
//! ## Why handles instead of raw pointers?
//! - No `unsafe` required at the JS ↔ WASM boundary
//! - [`BTreeMap`] guarantees deterministic iteration order — essential for
//!   reproducible WASM guests (hash-map randomisation would break it)
//! - Objects can be revoked by removing the handle with no GC pressure

use std::collections::BTreeMap;

/// A handle-based store for objects of type `T`.
///
/// Handles are monotonically incrementing 64-bit integers formatted as
/// zero-padded 16-character lowercase hex strings, so lexicographic order
/// equals insertion order.
pub struct Store<T> {
    next_id: u64,
    objects: BTreeMap<u64, T>,
}

impl<T> Store<T> {
    /// Create an empty store.
    pub fn new() -> Self {
        Self { next_id: 0, objects: BTreeMap::new() }
    }

    /// Insert `obj` and return its opaque handle.
    pub fn insert(&mut self, obj: T) -> String {
        let id = self.next_id;
        self.next_id += 1;
        self.objects.insert(id, obj);
        format!("{id:016x}")
    }

    /// Return a shared reference to the object behind `handle`, or `None`.
    pub fn get(&self, handle: &str) -> Option<&T> {
        let id = u64::from_str_radix(handle, 16).ok()?;
        self.objects.get(&id)
    }

    /// Return a mutable reference to the object behind `handle`, or `None`.
    pub fn get_mut(&mut self, handle: &str) -> Option<&mut T> {
        let id = u64::from_str_radix(handle, 16).ok()?;
        self.objects.get_mut(&id)
    }

    /// Remove and return the object behind `handle`, or `None`.
    pub fn remove(&mut self, handle: &str) -> Option<T> {
        let id = u64::from_str_radix(handle, 16).ok()?;
        self.objects.remove(&id)
    }

    /// Number of live objects in the store.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut store: Store<String> = Store::new();
        let h = store.insert("hello".to_string());
        assert_eq!(store.get(&h), Some(&"hello".to_string()));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn remove_returns_value() {
        let mut store: Store<u32> = Store::new();
        let h = store.insert(42);
        assert_eq!(store.remove(&h), Some(42));
        assert!(store.is_empty());
    }

    #[test]
    fn handles_are_monotonic_hex() {
        let mut store: Store<u8> = Store::new();
        let h0 = store.insert(0);
        let h1 = store.insert(1);
        let h2 = store.insert(2);
        // Lexicographic order == insertion order (zero-padded 16-char hex)
        assert!(h0 < h1);
        assert!(h1 < h2);
    }

    #[test]
    fn unknown_handle_returns_none() {
        let store: Store<u8> = Store::new();
        assert_eq!(store.get("deadbeefdeadbeef"), None);
    }
}
