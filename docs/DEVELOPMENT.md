# Development Guide

This guide covers development workflows, debugging, and common tasks for Stone contributors.

## Development Setup

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/stone-tauri.git
cd stone-tauri

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### IDE Setup

#### Visual Studio Code

Recommended extensions:
- rust-analyzer - Rust language support
- Tauri - Tauri development tools
- ESLint - JavaScript/TypeScript linting
- Prettier - Code formatting

Settings:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Project Structure

```
stone-tauri/
├── src/                        # Frontend (React + TypeScript)
│   ├── api/                   # Tauri IPC API layer
│   ├── components/            # React components
│   │   ├── base/             # Foundation UI components
│   │   ├── composites/       # Composite components
│   │   └── features/         # Feature-specific components
│   ├── hooks/                # Custom React hooks
│   ├── stores/               # Zustand state stores
│   ├── utils/                # Utility functions
│   └── main.tsx              # App entry point
│
├── src-tauri/                 # Backend (Rust)
│   ├── src/
│   │   ├── domain/           # Pure business logic
│   │   │   ├── entities/     # Business entities
│   │   │   ├── ports/        # Interface definitions
│   │   │   ├── services/     # Domain services
│   │   │   └── value_objects/# Value objects
│   │   ├── application/      # Use case implementations
│   │   │   └── usecases/
│   │   ├── adapters/         # External system connections
│   │   │   ├── inbound/      # Tauri commands, UI
│   │   │   └── outbound/     # Repositories, services
│   │   ├── infrastructure/   # Cross-cutting concerns
│   │   │   ├── config/       # Configuration
│   │   │   ├── container.rs  # Dependency injection
│   │   │   └── database/     # Database setup
│   │   └── shared/           # Shared code
│   ├── migrations/           # Diesel migrations
│   └── Cargo.toml
│
├── docs/                      # Documentation
│   ├── ARCHITECTURE.md       # Architecture guide
│   ├── CONTRIBUTING.md       # Contribution guidelines
│   └── DEVELOPMENT.md        # This file
│
└── .github/                   # GitHub configuration
    ├── ISSUE_TEMPLATE/       # Issue templates
    └── PULL_REQUEST_TEMPLATE.md
```

## Development Workflow

### Running the App

```bash
# Development mode with hot reload
pnpm tauri dev

# Build for production
pnpm tauri build

# The output will be in:
# macOS: src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/appimage/
```

### Frontend Development

```bash
# Start Vite dev server only
pnpm dev

# Build frontend
pnpm build

# Type check
pnpm typecheck

# Lint
pnpm lint
```

### Backend Development

```bash
# Check compilation
cargo check

# Build
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_name

# Lint
cargo clippy

# Format
cargo fmt

# Watch for changes and recompile
cargo watch -x check
```

## Database Management

### Diesel CLI

```bash
# Install Diesel CLI
cargo install diesel_cli --no-default-features --features sqlite

# Create new migration
diesel migration generate migration_name

# Run migrations
diesel migration run

# Rollback last migration
diesel migration revert

# Update schema.rs
diesel print-schema > src-tauri/src/shared/database/schema.rs
```

### Migration Example

```sql
-- migrations/YYYY-MM-DD-HHMMSS_add_column/up.sql
ALTER TABLE notes ADD COLUMN new_column TEXT;

-- migrations/YYYY-MM-DD-HHMMSS_add_column/down.sql
ALTER TABLE notes DROP COLUMN new_column;
```

After creating migration:
```bash
diesel migration run
diesel print-schema > src-tauri/src/shared/database/schema.rs
```

## Debugging

### Rust Backend

#### Using println! and dbg!

```rust
println!("Value: {:?}", value);
dbg!(&value);
```

#### Using tracing

```rust
use tracing::{info, warn, error, debug};

info!("Starting operation");
debug!("Debug info: {:?}", data);
warn!("Warning message");
error!("Error occurred: {}", err);
```

Run with logging:
```bash
RUST_LOG=debug pnpm tauri dev
RUST_LOG=stone_tauri=trace pnpm tauri dev
```

#### VS Code Debugger

1. Install CodeLLDB extension
2. Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Tauri Debug",
      "cargo": {
        "args": [
          "build",
          "--manifest-path=./src-tauri/Cargo.toml",
          "--no-default-features"
        ]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Frontend Debugging

#### Browser DevTools

```bash
# Enable DevTools in development
# Already enabled by default in Tauri dev mode
pnpm tauri dev
```

Then press `F12` or `Cmd+Option+I` to open DevTools.

#### React DevTools

Install React DevTools browser extension, it works with Tauri's webview.

#### Console Logging

```typescript
console.log('Value:', value);
console.error('Error:', error);
console.table(data);
```

#### VS Code Debugger

Add to `.vscode/launch.json`:

```json
{
  "type": "chrome",
  "request": "launch",
  "name": "Launch Chrome",
  "url": "http://localhost:1420",
  "webRoot": "${workspaceFolder}/src"
}
```

## Testing

### Unit Tests (Rust)

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test module_name::

# Run doctests
cargo test --doc
```

### Integration Tests (Rust)

Create files in `src-tauri/tests/`:

```rust
// tests/integration_test.rs
use stone_tauri::*;

#[tokio::test]
async fn test_integration() {
    // Test code
}
```

### Component Tests (React)

```bash
# Run tests (when implemented)
pnpm test

# Watch mode
pnpm test:watch
```

## Performance Profiling

### Backend Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile the app
cargo flamegraph --dev
```

### Frontend Profiling

Use Chrome DevTools Performance tab:
1. Open DevTools (F12)
2. Go to Performance tab
3. Click Record
4. Perform actions
5. Stop recording
6. Analyze flame graph

## Common Tasks

### Adding a New Feature

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed steps.

Quick checklist:
1. Define domain entity
2. Create ports (interfaces)
3. Implement use case
4. Implement repository
5. Add Tauri commands
6. Wire dependencies
7. Build UI

### Updating Dependencies

```bash
# Update Rust dependencies
cargo update

# Update npm dependencies
pnpm update

# Check for outdated packages
cargo outdated
pnpm outdated
```

### Database Schema Changes

1. Create migration: `diesel migration generate name`
2. Edit `up.sql` and `down.sql`
3. Run: `diesel migration run`
4. Update schema: `diesel print-schema > src-tauri/src/shared/database/schema.rs`
5. Update mappers if needed
6. Test migration rollback: `diesel migration revert`

### Adding a New Tauri Command

1. Define in command file:
```rust
// src-tauri/src/adapters/inbound/feature_commands.rs
#[tauri::command]
pub async fn do_something(
    state: State<'_, AppState>,
    param: String,
) -> Result<Response, String> {
    state.feature_usecases
        .do_something(param)
        .await
        .map_err(|e| e.to_string())
}
```

2. Register in `lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    feature_commands::do_something,
])
```

3. Create frontend API:
```typescript
// src/api/featureAPI.ts
export const featureAPI = {
  doSomething: (param: string): Promise<IpcResponse<Response>> =>
    invokeIpc('do_something', { param }),
};
```

## Troubleshooting

### Build Errors

**Problem:** Diesel schema out of sync
```
Solution: diesel migration run && diesel print-schema > src-tauri/src/shared/database/schema.rs
```

**Problem:** Tauri build fails
```
Solution: Clean and rebuild
cargo clean
rm -rf src-tauri/target
pnpm tauri build
```

### Runtime Errors

**Problem:** Database locked
```
Solution: Ensure no other instances are running, check file watchers
```

**Problem:** IPC command not found
```
Solution: Ensure command is registered in lib.rs invoke_handler
```

### Performance Issues

**Problem:** Slow startup
```
Solution: Check for expensive operations in setup_app(), consider lazy loading
```

**Problem:** High memory usage
```
Solution: Profile with instruments/perftools, check for memory leaks
```

## Useful Commands

```bash
# Clean build artifacts
cargo clean
rm -rf dist node_modules

# Reinstall dependencies
pnpm install

# Check bundle size
du -sh src-tauri/target/release/bundle/

# View database
sqlite3 ~/.stone/stone.db
> .tables
> .schema notes
> SELECT * FROM notes;

# Check Rust version
rustc --version
cargo --version

# Check Node version
node --version
pnpm --version
```

## Environment Variables

```bash
# Enable Rust logging
RUST_LOG=debug pnpm tauri dev
RUST_LOG=stone_tauri=trace pnpm tauri dev

# Custom database path
STONE_DB_PATH=/path/to/db pnpm tauri dev

# Diesel database URL
DATABASE_URL=sqlite://stone.db
```

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Diesel Documentation](https://diesel.rs/guides/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Architecture Guide](./ARCHITECTURE.md)
- [Contributing Guide](./CONTRIBUTING.md)
