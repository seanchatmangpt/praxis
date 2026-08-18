// FAIL (E0277): Implementing `Sealed` for an external type is rejected because
// `Sealed` has a private supertrait (`sealed::private::Sealed`) that cannot be
// named or implemented outside the `types` module.

use {{project-name}}::Sealed;

struct MySpy;

// E0277: the trait bound `MySpy: sealed::private::Sealed` is not satisfied
impl Sealed for MySpy {}

fn main() {}
