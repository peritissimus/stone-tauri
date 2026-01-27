# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2   | :x:                |

## Reporting a Vulnerability

We take the security of Stone seriously. If you discover a security vulnerability, please follow these steps:

### Private Disclosure

**Please do not publicly disclose the vulnerability until we've had a chance to address it.**

1. **Email**: Send details to the maintainers (create a security contact email)
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Regular Updates**: Every 7 days until resolved
- **Fix Timeline**: Varies by severity
  - Critical: 7 days
  - High: 30 days
  - Medium: 90 days
  - Low: Best effort

### Disclosure Policy

Once the vulnerability is fixed:
1. We'll create a security advisory
2. Credit will be given to the reporter (unless anonymity is requested)
3. Release notes will include security fixes

## Security Best Practices

### For Users

- **Keep Updated**: Always use the latest version
- **Local First**: Stone stores data locally, no cloud by default
- **File Permissions**: Ensure your workspace directory has appropriate permissions
- **Git Integration**: Be cautious when initializing Git in workspaces with sensitive data

### For Developers

- **Input Validation**: Validate all user input at the domain layer
- **SQL Injection**: Use Diesel's query builder (never raw SQL with user input)
- **Path Traversal**: Validate file paths before filesystem operations
- **XSS Prevention**: Sanitize HTML when rendering user content
- **Dependency Updates**: Regularly update dependencies for security patches

## Known Security Considerations

### Local Storage

Stone stores all data locally:
- **Database**: SQLite database in `.stone/stone.db`
- **Files**: Markdown files in workspace directory
- **No Encryption**: Data is not encrypted at rest by default

**Recommendation**: Use full-disk encryption (FileVault, BitLocker, LUKS) for sensitive data.

### Git Integration

When using Git features:
- Commits may contain metadata (author, timestamps)
- Pushing to remote repositories exposes data to that service
- Review `.gitignore` before initializing Git in a workspace

### File Watcher

Stone watches your workspace for changes:
- Only monitors the workspace directory
- Does not access files outside the workspace
- File watcher runs with your user permissions

## Security Features

### Tauri Security

Stone benefits from Tauri's security model:
- **IPC Sandboxing**: Frontend can only call explicitly exposed commands
- **Content Security Policy**: Restricts what the webview can load
- **No Node.js Runtime**: No access to Node.js APIs from frontend
- **System Webview**: Uses platform's secure webview (not bundled Chromium)

### Rust Memory Safety

Rust provides memory safety guarantees:
- No buffer overflows
- No use-after-free
- No data races (in safe code)

## Security Audit

No formal security audit has been performed yet. If you're interested in conducting or funding a security audit, please reach out.

## Additional Resources

- [Tauri Security Documentation](https://tauri.app/v1/references/architecture/security/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
