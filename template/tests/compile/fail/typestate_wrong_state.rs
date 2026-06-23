// FAIL (E0599): Calling transition() on State<Verified> — no such method.
//
// The typestate machine only allows `transition()` on `State<Pending>`.
// Calling it on a `State<Verified>` is a compile-time error because no such
// method is defined for that type parameter.

use {{project-name}}::{Pending, State};

fn main() {
    let pending = State::<Pending>::new();
    let verified = pending.transition();   // ok: Pending → Verified

    // E0599: no method named `transition` found for struct `State<Verified>`
    let _double = verified.transition();
}
