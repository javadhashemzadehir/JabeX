# Security Policy

JabeX is a **private, offline-first** desktop application. Core study data stays on the user's device. We take security and privacy seriously.

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅ Yes    |
| < 0.1   | ❌ No     |

Only the latest published minor line of the current major version receives security fixes unless otherwise announced.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report privately using one of:

1. **GitHub Private Vulnerability Reporting** — use *Security → Report a vulnerability* on the repository (preferred when enabled)
2. **Email** — contact the maintainers listed in the repository profile / `CODEOWNERS` with subject `[JabeX Security]`

Include as much detail as you can:

- Affected version / commit
- OS and architecture
- Steps to reproduce
- Impact (data exposure, code execution, privilege escalation, etc.)
- Proof of concept (if available)

We aim to acknowledge reports within **7 days** and provide a status update within **14 days**.

## Security Expectations

- No analytics, telemetry, or tracking SDKs in the core product
- No required cloud accounts for studying or storing cards
- User data must not leave the device unless the user explicitly opts into a future export/share feature
- Tauri capabilities follow least privilege
- IPC inputs are validated (Rust + Zod on the TypeScript boundary)
- Secrets, tokens, and user database files must never be committed

## Out of Scope (examples)

- Issues that require physical access to an unlocked machine
- Social engineering of end users
- Vulnerabilities in third-party dependencies already fixed in a newer release (please open a Dependabot / dependency PR instead when possible)

## Disclosure

We prefer coordinated disclosure. Please allow reasonable time for a fix and release before publishing details. Credit will be given to reporters who wish to be named, unless they prefer anonymity.
