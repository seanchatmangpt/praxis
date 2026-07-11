use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use serde::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    // If bumpalo collections implement Deserialize, how does it work?
    // It doesn't, because it needs the Bump.
}
