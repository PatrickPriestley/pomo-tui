# Security Policy

## Supported Versions

Currently supporting security updates for:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in pomo-tui, please follow these steps:

1. **DO NOT** open a public issue
2. Email the details to: pom.tui@priestley.io
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Fix Timeline**: Depends on severity
  - Critical: Within 72 hours
  - High: Within 1 week
  - Medium: Within 2 weeks
  - Low: Next release

## Security Best Practices

When using pomo-tui:

1. **Database Security**: The SQLite database is stored locally. Ensure proper file permissions.
2. **Environment Variables**: Never commit `.env` files with sensitive data.
3. **Updates**: Keep pomo-tui updated to receive security patches.

## Disclosure Policy

- Security issues will be disclosed publicly after a fix is available
- Credit will be given to reporters (unless anonymity is requested)
- A security advisory will be published on GitHub

## Contact

For security concerns, please contact:
- Primary: [Create a security advisory](https://github.com/PatrickPriestley/pomo-tui/security/advisories/new)
- Email: pom.tui@priestley.io

Thank you for helping keep pomo-tui secure!