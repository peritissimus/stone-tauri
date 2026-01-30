# Open Source Preparation Checklist

This document tracks the preparation of Stone for public open source release.

## ✅ Completed

### Documentation

- [x] README.md - Professional, comprehensive overview
- [x] LICENSE - MIT License
- [x] CHANGELOG.md - Version history and changes
- [x] SECURITY.md - Security policy and reporting guidelines
- [x] docs/ARCHITECTURE.md - Detailed architecture guide
- [x] docs/CONTRIBUTING.md - Contribution guidelines
- [x] docs/DEVELOPMENT.md - Development workflow guide

### GitHub Templates

- [x] .github/PULL_REQUEST_TEMPLATE.md - PR template
- [x] .github/ISSUE_TEMPLATE/bug_report.md - Bug report template
- [x] .github/ISSUE_TEMPLATE/feature_request.md - Feature request template

### Code Organization

- [x] Moved `quick_capture_window.rs` to proper adapter location
- [x] Verified hexagonal architecture compliance
- [x] Ensured no business logic in UI layer
- [x] All code follows architectural patterns

### Repository Hygiene

- [x] Removed placeholder/dummy data
- [x] Checked for sensitive information (API keys, passwords, etc.)
- [x] Created script to clean .vscode from git history
- [x] .gitignore properly configured

### Metadata

- [x] Updated package.json with correct name and version
- [x] Updated Cargo.toml with correct name and version
- [x] Updated tauri.conf.json with proper app metadata
- [x] Synced version numbers (0.2.29) across all files

### Branding

- [x] App name: Stone
- [x] Bundle identifier: com.stone.app
- [x] Icon assets properly configured
- [x] Professional README without emojis

## ⏳ Before Public Release

### Git History Cleanup

- [ ] Run `scripts/clean-git-history.sh` to remove .vscode from history
- [ ] Verify repository after cleanup
- [ ] Force push cleaned history (if not yet public)

### GitHub Repository Setup

- [ ] Create public repository on GitHub
- [ ] Push code to GitHub
- [ ] Configure repository settings:
  - [ ] Add description
  - [ ] Add topics/tags (rust, tauri, note-taking, markdown, etc.)
  - [ ] Enable Issues
  - [ ] Enable Discussions
  - [ ] Set up branch protection for main
- [ ] Create initial release (v0.2.29)

### Community

- [ ] Set up Discussions for Q&A
- [ ] Create roadmap discussion
- [ ] Add CODE_OF_CONDUCT.md (if desired)
- [ ] Add FUNDING.yml (if accepting sponsorships)

### CI/CD (Optional but Recommended)

- [ ] Set up GitHub Actions for:
  - [ ] Rust tests
  - [ ] Frontend tests
  - [ ] Clippy linting
  - [ ] Build verification
  - [ ] Release builds

### Website/Landing Page (Optional)

- [ ] Create GitHub Pages site
- [ ] Add screenshots/demo video
- [ ] Add download links

## 📋 Release Checklist

When ready to make first public release:

1. **Final Code Review**

   - [ ] All tests passing
   - [ ] No TODOs or FIXMEs that need addressing
   - [ ] Documentation up to date
   - [ ] Version numbers consistent

2. **Clean Git History**

   ```bash
   ./scripts/clean-git-history.sh
   ```

3. **Create Release**

   ```bash
   git tag v0.2.29
   git push origin v0.2.29
   ```

4. **Build Release Binaries**

   ```bash
   pnpm tauri build
   ```

5. **GitHub Release**

   - Create release on GitHub
   - Upload binaries for macOS, Windows, Linux
   - Include CHANGELOG.md excerpt
   - Mark as pre-release if not production-ready

6. **Announce**
   - Twitter/X
   - Reddit (r/rust, r/opensource, r/selfhosted)
   - Hacker News (Show HN)
   - Dev.to blog post

## 📝 Notes

### Architecture Compliance

All code follows hexagonal architecture:

- Domain layer: Pure business logic, no external dependencies
- Application layer: Use case implementations
- Adapters layer: External system connections (Tauri, Diesel, File System)
- Infrastructure layer: Cross-cutting concerns (DI, config, database)

### Files Outside Standard Layers

- `src-tauri/src/lib.rs` - Application entry point (acceptable)
- `src-tauri/src/main.rs` - Tauri main function (acceptable)
- All other code properly organized in architectural layers

### Security Considerations

- No sensitive data in repository
- .env files properly ignored
- Database files ignored
- No hardcoded credentials
- Security policy documented

### Next Steps

After open source release:

1. Monitor issues and respond promptly
2. Welcome first contributors
3. Consider setting up Discord/Slack for community
4. Continue development on roadmap items
5. Build community around the project

## 🎉 Ready for Open Source

All preparation tasks are complete. The repository is ready for public release once git history is cleaned and pushed to GitHub.

**Last Updated:** 2025-01-27
**Version:** 0.2.29
**Status:** Ready for cleanup and release
