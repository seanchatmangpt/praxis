use bumpalo::Bump;
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use serde::Deserialize;

#[derive(Deserialize)]
struct Test<'a> {
    name: BumpString<'a>,
    items: BumpVec<'a, BumpString<'a>>,
}

fn main() {
    println!("Compiles!");
}
