import Lake
open Lake DSL

package «praxis-lean-pilot» where
  -- Add mathlib in a second lane when needed.

@[default_target]
lean_lib Praxis where
  roots := #[`Praxis]
