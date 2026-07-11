use bumpalo::collections::Vec as BumpVec;
use serde::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    name: &'a str,
    #[serde(borrow)]
    items: BumpVec<'a, &'a str>,
}
