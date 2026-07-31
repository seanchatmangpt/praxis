#!/usr/bin/env python3
"""Stable entrypoint for the ggen code-inventory builder.

`ggen.lock` records the rendered pack graph and therefore changes as a
consequence of this inventory. It cannot also be an input to the inventory
content hash without creating a self-invalidating cycle. This wrapper excludes
that projection-control artifact for both build and verification, then delegates
to the governed builder module.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

ROOT = Path(__file__).resolve().parents[1]
BUILDER = ROOT / "scripts" / "build-ggen-code-inventory.py"


def load_builder() -> ModuleType:
    spec = importlib.util.spec_from_file_location("praxis_ggen_inventory_builder", BUILDER)
    if spec is None or spec.loader is None:
        raise SystemExit(
            "ggen-code-inventory: REFUSED: unable to load governed inventory builder"
        )
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    module.EXCLUDED_EXACT.add("ggen.lock")
    return module


def main() -> int:
    module = load_builder()
    if len(sys.argv) != 2 or sys.argv[1] not in {"build", "verify"}:
        print("usage: ggen-code-inventory.py {build|verify}", file=sys.stderr)
        return 2
    return module.build() if sys.argv[1] == "build" else module.verify()


if __name__ == "__main__":
    raise SystemExit(main())
