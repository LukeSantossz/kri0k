# Security Policy

## Reporting Security Vulnerabilities

Kri0K is a security research tool designed to assist in authorized penetration testing. We take security seriously, both in the tool's implementation and in how it should be used.

### Responsible Disclosure

If you discover a security vulnerability in Kri0K itself (e.g., bypass of safeguards, scope validation issues, audit log tampering), please report it responsibly:

**Email:** <!-- TODO(KRK-DISCLOSURE): Add security contact email -->

Please **do not** open public GitHub issues for security vulnerabilities until we've had a chance to address them.

### Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial Assessment:** Within 7 days
- **Fix Timeline:** Depends on severity (critical issues within 30 days)

## Security Model

Kri0K implements multiple layers of defense-in-depth to prevent misuse:

- **Scope Validation:** All targets are validated against scope.yaml before execution (M-01, M-02)
- **Propose-Only Mode:** Default mode requires human approval for all TTPs (M-05, M-15)
- **Audit Logging:** All operations are logged in an append-only audit trail (M-13, M-22)
- **Kill Switch:** Emergency stop mechanism accessible at all times (M-36)
- **Human Gates:** High-risk TTPs require explicit human approval (M-21)

For complete details, see:
- [Threat Model](docs/security/THREAT_MODEL.md)
- [Architecture Decision Records](docs/adr/)

## Ethical Use

Kri0K must only be used for **authorized** security testing. Unauthorized use may be illegal and unethical. Always:

1. Obtain written authorization before testing any system
2. Respect the defined scope and engagement boundaries
3. Follow responsible disclosure practices for findings
4. Comply with all applicable laws and regulations

See `LICENSE` for additional terms and the ethical use clause.

## Security Features Status

| Feature | Status | ADR |
|---------|--------|-----|
| Scope Validation | Stub (T7) | ADR-0011 |
| Audit Logging | Stub (T7) | ADR-0007 |
| Propose-Only Mode | Implemented | ADR-0006 |
| TTP Trait | Stub (T7) | ADR-0012 |
| Kill Switch | Implemented | - |
| Human Gates | Stub (T7) | - |

MVP-0 includes **stubs** for critical safeguards. Full implementation is tracked in milestone T7.
