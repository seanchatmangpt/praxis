# Signing-key rotation (v26.9.24)

Recorded 2026-09-24 (fleet key scan after the single-repo migration). Base `cd6bd52e784b` of `praxis`.
Every private key listed here was committed to this repository and is therefore compromised: every receipt or
attestation signed with it carries no signing authority (standing REFUSED, broken_term R_missing_authority).
The keys leave the tree (history is not rewritten; no force-push), and each key directory's `.gitignore` now
covers both halves. Every checkout keeps its own pair: ggen generates one on first use, and a tracked public
half without its private half would make that first `ggen sync` refuse [FM-KEY-010/011]. The canonical
checkout's new public key is published below for anyone verifying its future receipts.

| key dir | removed private key sha256 | removed public key sha256 | new public key (canonical checkout) |
|---|---|---|---|
| `.ggen/keys` | `447e816911667a45102ebd3746c77502de24e23c1a27e29e95b084f1a604495f` | `b16fe50d66a270878865953eb2d0f4632d4a7fa03bf0f1458427e24db6d1ec42` | `51ff80b1bcec8a638c99535c29bb91078093ff22fa1e81f02738afc793e1fd8a` |
| `.praxis/keys` | `9d37225d61e306cc9ed87ad51035d80681ef2daff4586accfde0b80dae6ca9ae` | `f074403025b69e7a1a3453f421142e36a6f128af63d7b58f71fdd92d8b7db35f` | `2e048565ae26808ad6d321b6f0861966e7ab296e1a0effcbc00d42e3e7beccdf` |
