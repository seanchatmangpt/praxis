//! Type-safe input validation for CLI arguments.
//!
//! The `ValidatedInput<T, V>` pattern moves validation from runtime to the type system,
//! ensuring "input has passed all gates" is explicit at construction time.

use std::marker::PhantomData;
use crate::error::{AppError, Result};

/// A validated input: `T` has passed all checks defined by validator `V`.
///
/// Construction fails if validation fails, so the only way to obtain a
/// `ValidatedInput<T, V>` is to pass `T` through `V::validate()`.
///
/// # Example
///
/// ```rust
/// use {{project-name}}::validation::{ValidatedInput, FileExistsValidator};
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("/etc/hosts");
/// let validated = ValidatedInput::<PathBuf, FileExistsValidator>::new(path)?;
///
/// // Now we can use the validated input safely
/// let file_path = validated.inner();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ValidatedInput<T, V: Validator> {
    value: T,
    _validator: PhantomData<V>,
}

impl<T, V: Validator<Input = T>> ValidatedInput<T, V> {
    /// Validate `value` using the validator `V`. Returns `Ok` only if validation succeeds.
    pub fn new(value: T) -> Result<Self> {
        V::default().validate(&value)?;
        Ok(Self {
            value,
            _validator: PhantomData,
        })
    }

    /// Access the validated inner value by reference.
    pub fn inner(&self) -> &T {
        &self.value
    }

    /// Consume the wrapper and extract the validated inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Clone, V: Validator<Input = T>> Clone for ValidatedInput<T, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _validator: PhantomData,
        }
    }
}

impl<T: std::fmt::Debug, V: Validator> std::fmt::Debug for ValidatedInput<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatedInput")
            .field("value", &self.value)
            .field("validator", &std::any::type_name::<V>())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Validator trait
// ---------------------------------------------------------------------------

/// Strategy for validating inputs of type `T`.
///
/// Validators are stateless; validation rules are defined in the `validate` method.
///
/// # Example
///
/// ```rust
/// use {{project-name}}::validation::Validator;
/// use {{project-name}}::error::Result;
///
/// struct PositiveIntValidator;
///
/// impl Validator for PositiveIntValidator {
///     type Input = i32;
///
///     fn validate(&self, value: &i32) -> Result<()> {
///         if *value <= 0 {
///             return Err({{project-name}}::error::AppError::validation(
///                 "PositiveIntValidator: value must be > 0",
///             ));
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait Validator: Default + Send + Sync {
    /// The input type this validator checks.
    type Input: Sized;

    /// Validate the input. Return `Ok(())` if valid, `Err` with context otherwise.
    fn validate(&self, input: &Self::Input) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Built-in validators
// ---------------------------------------------------------------------------

/// Validates that a file path exists on the filesystem.
pub struct FileExistsValidator;

impl Default for FileExistsValidator {
    fn default() -> Self {
        Self
    }
}

impl Validator for FileExistsValidator {
    type Input = std::path::PathBuf;

    fn validate(&self, path: &std::path::PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(AppError::validation(format!(
                "file does not exist: {}",
                path.display()
            )));
        }
        Ok(())
    }
}

/// Validates that a string is non-empty.
pub struct NonEmptyStringValidator;

impl Default for NonEmptyStringValidator {
    fn default() -> Self {
        Self
    }
}

impl Validator for NonEmptyStringValidator {
    type Input = String;

    fn validate(&self, s: &String) -> Result<()> {
        if s.is_empty() {
            return Err(AppError::validation("string must not be empty"));
        }
        Ok(())
    }
}

/// Validates that an integer is within a range `[min, max]`.
pub struct RangeValidator {
    pub min: i32,
    pub max: i32,
}

impl RangeValidator {
    pub fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

impl Default for RangeValidator {
    fn default() -> Self {
        Self { min: 0, max: 1000 }
    }
}

impl Validator for RangeValidator {
    type Input = i32;

    fn validate(&self, value: &i32) -> Result<()> {
        if *value < self.min || *value > self.max {
            return Err(AppError::validation(format!(
                "value must be in range [{}, {}], got {}",
                self.min, self.max, value
            )));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Clap integration helper
// ---------------------------------------------------------------------------

/// Helper for integrating `ValidatedInput` with Clap via `value_parser`.
///
/// # Example
///
/// ```rust,ignore
/// use {{crate_name}}::validation::{ValidatedInput, FileExistsValidator};
/// use clap::Args;
/// use std::path::PathBuf;
///
/// #[derive(Args)]
/// pub struct MyVerbArgs {
///     #[arg(value_parser = clap_validated_input_parser::<PathBuf, FileExistsValidator>)]
///     pub file: ValidatedInput<PathBuf, FileExistsValidator>,
/// }
/// ```
///
/// This requires custom implementation due to Clap's value_parser trait,
/// but the pattern is straightforward in practice.
pub fn clap_validated_input_parser<T: std::str::FromStr, V: Validator<Input = T>>(
    s: &str,
) -> std::result::Result<ValidatedInput<T, V>, String>
where
    T::Err: std::fmt::Display,
{
    let value = T::from_str(s).map_err(|e| e.to_string())?;
    ValidatedInput::new(value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn validated_input_accepts_valid() {
        let path = std::path::PathBuf::from("/etc/hosts");
        // This test assumes /etc/hosts exists on the system
        if path.exists() {
            let result = ValidatedInput::<_, FileExistsValidator>::new(path.clone());
            assert!(result.is_ok());
            assert_eq!(result.unwrap().inner(), &path);
        }
    }

    #[test]
    fn validated_input_rejects_invalid_file() {
        let path = std::path::PathBuf::from("/nonexistent/path/to/file.txt");
        let result = ValidatedInput::<_, FileExistsValidator>::new(path);
        assert!(result.is_err());
    }

    #[test]
    fn non_empty_string_validator() {
        let result = ValidatedInput::<_, NonEmptyStringValidator>::new(String::new());
        assert!(result.is_err());

        let result = ValidatedInput::<_, NonEmptyStringValidator>::new("hello".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn range_validator_accepts_in_range() {
        let validator = RangeValidator::new(1, 10);
        assert!(validator.validate(&5).is_ok());
        assert!(validator.validate(&1).is_ok());
        assert!(validator.validate(&10).is_ok());
    }

    #[test]
    fn range_validator_rejects_out_of_range() {
        let validator = RangeValidator::new(1, 10);
        assert!(validator.validate(&0).is_err());
        assert!(validator.validate(&11).is_err());
    }

    #[test]
    fn validated_input_into_inner() {
        let result = ValidatedInput::<_, NonEmptyStringValidator>::new("test".to_string()).unwrap();
        let inner = result.into_inner();
        assert_eq!(inner, "test");
    }

    #[test]
    fn validated_input_clone() {
        let result = ValidatedInput::<_, NonEmptyStringValidator>::new("test".to_string()).unwrap();
        let cloned = result.clone();
        assert_eq!(cloned.inner(), "test");
    }
}
