# Security Policy

## Supported versions

Only the latest published release is supported with security fixes.

| Version | Supported |
| ------- | --------- |
| latest release on [Releases](https://github.com/ZengZichao/PhyloPanel-Windows/releases) | ✅ |
| older releases | ❌ |

## Reporting a vulnerability

Please do **not** open a public issue for security reports.

Use GitHub's private vulnerability reporting:
**Security → Report a vulnerability** on this repository, or open a private
advisory at https://github.com/ZengZichao/PhyloPanel-Windows/security/advisories/new

Include, if possible:

- The PhyloPanel version (Help → About, or the release tag)
- Windows version affected
- Steps to reproduce or a proof of concept

You can expect an initial response within 7 days. Fixes are released as soon as
practical, and reporters are credited in the release notes unless they prefer
otherwise.

## Scope

PhyloPanel embeds and drives third-party CLIs (GoTree by default, or any cobra
binary the user points it at) as child processes. Vulnerabilities in those
upstream tools should be reported to their own projects; issues in how
PhyloPanel invokes or exposes them are in scope here.

## Release integrity

Release assets ship with a `.sha256` checksum file. Verify a download on
Windows with:

```powershell
Get-FileHash .\PhyloPanel-<version>-win-x64.exe -Algorithm SHA256
```

and compare against the published `.sha256` file. Releases are not code-signed;
SmartScreen may warn on first launch.
