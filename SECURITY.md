# Security Policy

## Supported Versions

MarsDB is pre-1.0 — only the latest published release is supported.
Security fixes land as a new patch/minor release, not backported to
older tags.

| Version | Supported |
|---|---|
| latest (see [releases](https://github.com/knoguchi/marsdb/releases)) | ✅ |
| older | ❌ |

## Reporting a Vulnerability

**Do not open a public GitHub issue for a security vulnerability.**

Instead, use GitHub's private reporting:
[Security → Report a vulnerability](https://github.com/knoguchi/marsdb/security/advisories/new)
on this repository, or email tokyo246@gmail.com directly.

Include:
- A description of the vulnerability and its potential impact
- Steps to reproduce (a minimal Cypher query/program is ideal)
- The MarsDB version/commit affected

You should receive an acknowledgment within a few days. Once a fix is
ready, it will be released and credited (unless you prefer to stay
anonymous) in the release notes / `CHANGELOG.md`.

## Scope

MarsDB is an embedded, in-process database — it has no network listener
or server component of its own. The most relevant attack surface is the
**Cypher parser** (`marsdb-query`), the one part of the library that
takes raw, untrusted string input directly when a host application
passes user-supplied text as a query. It's fuzzed via `cargo-fuzz` (see
`CONTRIBUTING.md`); a crash or panic there is a real bug worth
reporting. `$parameter` values are not parsed as Cypher syntax and are a
much smaller surface by construction.
