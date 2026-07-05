import Lake
open Lake DSL

package «praxis-lean-pilot» where

-- Pinned to the `v4.31.0` tag, which matches this environment's installed
-- toolchain (leanprover/lean4:v4.31.0) exactly -- avoids also having to
-- fetch a second, different Lean toolchain just to satisfy Mathlib's own
-- lean-toolchain requirement (Mathlib's `master` branch currently targets
-- a newer, different toolchain).
require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.31.0"

@[default_target]
lean_lib Praxis where
  roots := #[`Praxis]
