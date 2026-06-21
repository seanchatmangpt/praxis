# Broadening repo access

This kit was built in a Claude Code (web) session scoped to a single repo:
`seanchatmangpt/affidavit`. In that session Claude could:

- **read/write** `affidavit` (commit + push), and
- **clone public repos read-only** over HTTPS (enough to *survey* them),

but it **could not**:

- read or write the other repos through the GitHub tools (out of scope), or
- clone the **8 private** repos (no credentials): `knhk`, `kcura`, `kgold`,
  `chatmangpt`, `unibit`, `mcpp`, `stpnt`, `tower-lsp-composition`.

So the actual refactor of the other repos needs broader access. Options,
easiest first:

## 1. One session per repo (simplest)
Start a Claude Code session **from the target repo** (on the web, pick that
repo; in the CLI, run inside a checkout). That repo becomes the in-scope,
writable repo. Then:
```bash
git clone https://github.com/seanchatmangpt/praxis /tmp/rb
/tmp/rb/apply.sh .
# do the manual Cargo.toml steps, then open a PR
```

## 2. Add repos to an existing session
If your environment exposes repo-management tools (`list_repos` / `add_repo`),
ask Claude to list available repos and add the ones you want; they become
readable/writable in the same session. Availability depends on how the
environment was provisioned.

## 3. Reconfigure the environment
Recreate the Claude Code on the web environment with the repositories (and the
network policy) you need. Docs:
<https://code.claude.com/docs/en/claude-code-on-the-web>

## Doing it locally (works today, no session changes — and the only path for the private repos)
```bash
gh repo clone seanchatmangpt/<repo> && cd <repo>
/path/to/praxis/apply.sh .
git switch -c chore/house-style
git add -A && git commit -m "chore: adopt house Rust boilerplate"
gh pr create --fill
```
You have credentials for the private repos; the session does not — so the
private eight have to go through this local path (or option 1 from their own
session).
