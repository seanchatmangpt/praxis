//! Queryable verb registry using linkme distributed slices.
//!
//! This module makes verb metadata discoverable at runtime without central dispatch,
//! enabling introspection, testing, and mock registries.

use std::fmt;

/// Metadata for a single verb (CLI subcommand).
///
/// Verbs are discovered via linkme distributed slices and can be queried at runtime.
#[derive(Debug, Clone)]
pub struct VerbMetadata {
    /// Stable verb name (e.g., "verify", "assemble", "emit").
    pub name: &'static str,
    /// Human-readable description from the clap `about` field.
    pub description: &'static str,
    /// Verb handler function pointer (for potential dynamic dispatch).
    pub handler: VerbHandler,
}

impl fmt::Display for VerbMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

/// Opaque handler for a verb. Can be used for reflection or dynamic dispatch.
#[derive(Debug, Clone, Copy)]
pub struct VerbHandler(*const ());

unsafe impl Send for VerbHandler {}
unsafe impl Sync for VerbHandler {}

/// Registry of all available verbs (linkme distributed slice).
///
/// Verbs register themselves via:
/// ```ignore
/// #[linkme::distributed_slice(crate::discovery::VERBS)]
/// static MY_VERB: VerbMetadata = VerbMetadata {
///     name: "my-verb",
///     description: "Does something interesting",
///     handler: VerbHandler(my_verb_handler as *const ()),
/// };
/// ```
#[linkme::distributed_slice]
pub static VERBS: [VerbMetadata] = [..];

/// Query the verb registry.
///
/// # Example
///
/// ```no_run
/// use {{crate_name}}::discovery::VerbRegistry;
///
/// let registry = VerbRegistry::new();
///
/// // List all verbs
/// for verb in registry.list() {
///     println!("{}: {}", verb.name, verb.description);
/// }
///
/// // Find a specific verb
/// if let Some(verb) = registry.find("verify") {
///     println!("Found: {}", verb);
/// }
/// ```
pub struct VerbRegistry {
    verbs: &'static [VerbMetadata],
}

impl VerbRegistry {
    /// Create a new registry that discovers verbs from the linkme slice.
    pub fn new() -> Self {
        Self { verbs: &VERBS }
    }

    /// List all registered verbs in discovery order.
    pub fn list(&self) -> impl Iterator<Item = &'static VerbMetadata> {
        self.verbs.iter()
    }

    /// Count registered verbs.
    pub fn len(&self) -> usize {
        self.verbs.len()
    }

    /// Return `true` if no verbs are registered.
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    /// Find a verb by name (case-sensitive).
    pub fn find(&self, name: &str) -> Option<&'static VerbMetadata> {
        self.verbs.iter().find(|v| v.name == name)
    }

    /// Collect all verbs as a vector.
    pub fn collect(&self) -> Vec<&'static VerbMetadata> {
        self.verbs.iter().collect()
    }

    /// Check if a verb exists by name.
    pub fn contains(&self, name: &str) -> bool {
        self.find(name).is_some()
    }
}

impl Default for VerbRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_queryable() {
        let registry = VerbRegistry::new();
        // Verbs will be registered when this module is linked into a binary
        // In pure unit tests, the slice may be empty.
        let _ = registry.len();
        let _ = registry.list().next();
    }

    #[test]
    fn verb_metadata_displays() {
        let verb = VerbMetadata {
            name: "test",
            description: "A test verb",
            handler: VerbHandler(std::ptr::null()),
        };
        let s = format!("{}", verb);
        assert!(s.contains("test"));
        assert!(s.contains("A test verb"));
    }
}
