#!/usr/bin/env python3
"""Materialize the exact-pinned Chatman source graph required by Praxis.

This is a source transport/admission helper, not an execution or standing
oracle. It never promotes a sibling repository above UNKNOWN. It refuses to
overwrite dirty checkouts and emits a machine-readable materialization report.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib
from typing import Any


class Refused(RuntimeError):
    """A typed refusal to mutate an unsafe or contradictory local checkout."""


def run(*args: str, cwd: Path | None = None, sudo: bool = False) -> str:
    command = list(args)
    if sudo:
        command.insert(0, "sudo")
    completed = subprocess.run(
        command,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return completed.stdout.strip()


def git_head(path: Path) -> str:
    return run("git", "rev-parse", "HEAD", cwd=path)


def ensure_checkout(parent: Path, spec: dict[str, Any]) -> dict[str, Any]:
    name = spec["name"]
    target = parent / name
    wanted = spec["sha"]

    if target.exists():
        if not (target / ".git").exists():
            raise Refused(f"REFUSED:ECOSYSTEM_TARGET_NOT_GIT:{target}")
        dirty = run("git", "status", "--porcelain", cwd=target)
        if dirty:
            raise Refused(f"REFUSED:ECOSYSTEM_CHECKOUT_DIRTY:{target}")
        if git_head(target) != wanted:
            run("git", "fetch", "--depth", "1", "origin", wanted, cwd=target)
            run("git", "checkout", "--detach", wanted, cwd=target)
    else:
        run("git", "clone", "--filter=blob:none", "--no-checkout", spec["url"], str(target))
        run("git", "fetch", "--depth", "1", "origin", wanted, cwd=target)
        run("git", "checkout", "--detach", wanted, cwd=target)

    observed = git_head(target)
    if observed != wanted:
        raise Refused(
            f"REFUSED:ECOSYSTEM_SOURCE_IDENTITY_MISMATCH:{name}:{wanted}:{observed}"
        )

    missing = [rel for rel in spec.get("required_paths", []) if not (target / rel).exists()]
    if missing:
        raise Refused(
            f"REFUSED:ECOSYSTEM_REQUIRED_PATH_MISSING:{name}:{','.join(missing)}"
        )

    return {
        "name": name,
        "url": spec["url"],
        "admitted_sha": wanted,
        "observed_sha": observed,
        "standing": spec.get("standing", "UNKNOWN"),
        "path": str(target),
        "required_paths_verified": list(spec.get("required_paths", [])),
    }


def mkdir_with_fallback(path: Path) -> None:
    try:
        path.mkdir(parents=True, exist_ok=True)
    except PermissionError:
        run("mkdir", "-p", str(path), sudo=True)


def legacy_alias(legacy_root: Path, target: Path) -> dict[str, str]:
    alias = legacy_root / target.name
    mkdir_with_fallback(legacy_root)

    if alias.exists() or alias.is_symlink():
        if alias.resolve() == target.resolve():
            return {"alias": str(alias), "target": str(target), "disposition": "existing"}
        if (alias / ".git").exists() and git_head(alias) == git_head(target):
            return {
                "alias": str(alias),
                "target": str(alias.resolve()),
                "disposition": "compatible-existing-checkout",
            }
        raise Refused(f"REFUSED:LEGACY_ALIAS_COLLISION:{alias}")

    try:
        alias.symlink_to(target, target_is_directory=True)
    except PermissionError:
        run("ln", "-s", str(target), str(alias), sudo=True)

    if alias.resolve() != target.resolve():
        raise Refused(f"REFUSED:LEGACY_ALIAS_MISMATCH:{alias}:{target}")
    return {"alias": str(alias), "target": str(target), "disposition": "created"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", type=Path, default=Path.cwd())
    parser.add_argument("--lock", type=Path)
    parser.add_argument("--legacy-root", type=Path, default=Path("/Users/sac"))
    parser.add_argument("--report", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    workspace = args.workspace.resolve()
    lock_path = (args.lock or workspace / ".chatmangpt/ecosystem.lock.toml").resolve()
    report_path = args.report or Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "praxis-chatman-ecosystem.json"

    if shutil.which("git") is None:
        raise Refused("BLOCKED:GIT_UNAVAILABLE")
    if not lock_path.exists():
        raise Refused(f"BLOCKED:ECOSYSTEM_LOCK_MISSING:{lock_path}")

    with lock_path.open("rb") as handle:
        lock = tomllib.load(handle)

    parent = workspace.parent
    repositories = []
    aliases = []
    for spec in lock.get("repository", []):
        evidence = ensure_checkout(parent, spec)
        repositories.append(evidence)
        aliases.append(legacy_alias(args.legacy_root, Path(evidence["path"])))

    report = {
        "schema_version": 1,
        "subject": str(workspace),
        "lock": str(lock_path),
        "observed": "exact source identities and required Cargo paths",
        "executed": "source materialization only",
        "not_executed": ["ggen", "Lean 4", "mfact", "BRCE", "GymAct"],
        "standing": "UNKNOWN",
        "repositories": repositories,
        "legacy_aliases": aliases,
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Refused as exc:
        print(str(exc), file=sys.stderr)
        sys.exit(2)
    except subprocess.CalledProcessError as exc:
        print(
            f"BLOCKED:ECOSYSTEM_TRANSPORT:{' '.join(exc.cmd)}:{exc.stderr.strip()}",
            file=sys.stderr,
        )
        sys.exit(exc.returncode or 1)
