// PASS: Pending → Verified typestate transition is legal.

use {{project-name}}::{Pending, State, Verified};

fn verify(s: State<Pending>) -> State<Verified> {
    s.transition()
}

fn main() {
    let pending = State::new();
    let _verified = verify(pending);
}
