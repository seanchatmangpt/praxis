#!/usr/bin/env python3
"""Stable entrypoint for the ggen code-inventory builder.

`ggen.lock` records the rendered pack graph and therefore changes as a
consequence of this inventory. It cannot also be an input to the inventory
content hash without creating a self-invalidating cycle. This wrapper excludes
that projection-control artifact for both build and verification, completes the
polyglot language/control vocabulary, then delegates to the governed builder.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

ROOT = Path(__file__).resolve().parents[1]
BUILDER = ROOT / "scripts" / "build-ggen-code-inventory.py"

ADDITIONAL_CODE_LANGUAGES = {
    ".asm": "Assembly",
    ".clj": "Clojure",
    ".cljs": "ClojureScript",
    ".cs": "C#",
    ".css": "CSS",
    ".dart": "Dart",
    ".fs": "F#",
    ".fsx": "F#",
    ".gleam": "Gleam",
    ".hcl": "HCL",
    ".hs": "Haskell",
    ".html": "HTML",
    ".jl": "Julia",
    ".jsx": "JavaScript",
    ".less": "Less",
    ".lhs": "Literate Haskell",
    ".lua": "Lua",
    ".ml": "OCaml",
    ".mli": "OCaml",
    ".move": "Move",
    ".nix": "Nix",
    ".php": "PHP",
    ".proto": "Protocol Buffers",
    ".r": "R",
    ".rb": "Ruby",
    ".s": "Assembly",
    ".scala": "Scala",
    ".scss": "SCSS",
    ".sol": "Solidity",
    ".sql": "SQL",
    ".svelte": "Svelte",
    ".swift": "Swift",
    ".tf": "Terraform",
    ".tfvars": "Terraform",
    ".vue": "Vue",
    ".zig": "Zig",
}

ADDITIONAL_CONTROL_NAMES = {
    "build.gradle",
    "build.gradle.kts",
    "build.zig",
    "deno.json",
    "deno.jsonc",
    "flake.lock",
    "flake.nix",
    "go.mod",
    "go.sum",
    "lakefile.lean",
    "mix.exs",
    "mix.lock",
    "package-lock.json",
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "poetry.lock",
    "pyproject.toml",
    "rebar.config",
    "requirements.txt",
    "settings.gradle",
    "settings.gradle.kts",
    "tsconfig.json",
    "yarn.lock",
}


def load_builder() -> ModuleType:
    spec = importlib.util.spec_from_file_location("praxis_ggen_inventory_builder", BUILDER)
    if spec is None or spec.loader is None:
        raise SystemExit(
            "ggen-code-inventory: REFUSED: unable to load governed inventory builder"
        )
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)

    # Projection/control artifacts cannot feed their own source-tree identity.
    module.EXCLUDED_EXACT.add("ggen.lock")

    # Complete the admitted executable and build-control vocabulary without
    # duplicating the builder implementation.
    module.CODE_LANGUAGES.update(ADDITIONAL_CODE_LANGUAGES)
    module.CONTROL_NAMES.update(ADDITIONAL_CONTROL_NAMES)
    return module


def main() -> int:
    module = load_builder()
    if len(sys.argv) != 2 or sys.argv[1] not in {"build", "verify"}:
        print("usage: ggen-code-inventory.py {build|verify}", file=sys.stderr)
        return 2
    return module.build() if sys.argv[1] == "build" else module.verify()


if __name__ == "__main__":
    raise SystemExit(main())
