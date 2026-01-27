# Changelog

All notable changes to Stone will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial public release preparation
- Comprehensive documentation (ARCHITECTURE.md, CONTRIBUTING.md, SECURITY.md)
- MIT License
- GitHub issue templates and PR template

## [0.2.29] - 2025-01-27

### Added
- Quick capture window with global hotkey (Alt+Space)
- Quick capture auto-refresh in editor when journal is open
- Event-driven architecture with EventPublisher
- Real-time sync between file system and editor
- Aerospace/yabai window manager compatibility
- Proper hexagonal architecture structure

### Fixed
- Quick capture window positioning on multi-monitor setups
- Window manager interference with quick capture window
- Event publishing to frontend (TokioEventPublisher with AppHandle)

### Changed
- App name from "stone-tauri" to "Stone"
- Bundle identifier to com.stone.app
- Version sync with original Electron version
- Improved README with premium styling
- Reorganized quick_capture_window.rs into proper adapter location

## [0.2.20] - 2025-01-17

### Added
- Complete Rust backend with hexagonal architecture
- Domain layer with pure business logic
- Application layer with use cases
- Adapters for Tauri commands and Diesel repositories
- Infrastructure layer with DI container

### Features
- Rich block-based editor (TipTap)
- Mermaid diagram support
- Syntax-highlighted code blocks (20+ languages)
- Task management with multiple states (TODO, DOING, DONE, etc.)
- Daily journal functionality
- Workspaces, folders, tags, notebooks
- File watcher with real-time sync
- Knowledge graph visualization
- Git integration (init, commit, history)
- Markdown-first storage
- Settings and preferences
- Dark mode with system awareness
- Recovery mode
- Command palette
- Bidirectional note linking

### Technical
- SQLite database with Diesel ORM
- Tokio async runtime
- File system integration
- Event broadcasting system
- ~20MB bundle size (90% smaller than Electron)

## [0.1.0] - 2025-01-12

### Added
- Initial Tauri setup
- Basic note CRUD operations
- SQLite database integration
- Frontend React app with Vite

[Unreleased]: https://github.com/yourusername/stone-tauri/compare/v0.2.29...HEAD
[0.2.29]: https://github.com/yourusername/stone-tauri/compare/v0.2.20...v0.2.29
[0.2.20]: https://github.com/yourusername/stone-tauri/compare/v0.1.0...v0.2.20
[0.1.0]: https://github.com/yourusername/stone-tauri/releases/tag/v0.1.0
