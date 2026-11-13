# Security

This project is unreleased. There is no supported stable desktop release.
No live hardware adapter or network inspector service is bundled. An opt-in
developer build exposes a private local Unix-socket inspector, including
JavaScript evaluation in the embedded view. Normal builds disable that endpoint.

## Reporting

Do not post credentials, access details, personal data or exploit payloads in a
public issue or pull request. Use private vulnerability reporting in the GitHub
Security tab when available. Otherwise request a private contact channel without
including sensitive details. Private reporting availability must be checked
before public launch; this policy does not imply that it is already enabled.

## Required boundaries

Keep developer evaluation opt-in
and disabled in ordinary release builds. Do not expose a debug socket or evaluator
on a network interface. Verify local access controls, bounded requests and method
allowlists before enabling inspection.

Hardware adapters and CLI commands must use user-supplied configuration, validate
their typed inputs and preserve provenance. Opening a project must not execute
commands, enable outputs, acquire credentials or establish remote access.

Diagnostic exports need review/redaction of messages, stack traces and paths.
Do not upload logs automatically. A schema is a contract, not authorization or
proof that its implementation enforces every constraint.

CI scans Git history and source content for secrets. Scan findings need review;
no scanner proves the absence of personal data or establishes legal clearance.
