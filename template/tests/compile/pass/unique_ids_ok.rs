// PASS: assert_unique_ids with a non-overlapping set compiles fine.

use {{project-name}}::assert_unique_ids;

const MY_IDS: &[&str] = &["dom:create", "dom:read", "dom:update", "dom:delete"];
const _: () = assert_unique_ids(MY_IDS);

fn main() {}
