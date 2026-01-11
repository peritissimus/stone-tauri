# Stone Tauri - Rust + Hexagonal Architecture

A native note-taking application built with **Tauri + Rust** following hexagonal architecture principles.

> 🎯 **Successfully Built & Running!** Database: `.stone/stone.db` • Dev server: `http://localhost:1420/`

## 🎯 What's Been Built

### ✅ Complete Backend (Rust)
- **Hexagonal Architecture** (Ports & Adapters pattern)
- **Domain Layer**: Pure business logic (Note entity, validation)
- **Application Layer**: Use cases (15+ operations)
- **Adapters**:
  - IN: Tauri commands (IPC)
  - OUT: Diesel ORM + SQLite repository
- **Infrastructure**: DI container, database setup
- **All 15+ Tauri commands implemented**

### ✅ Complete Frontend (React + TypeScript)
- Test UI with create, list, favorite, delete operations
- Real-time updates
- Error handling
- Loading states

### ✅ Database
- SQLite with Diesel ORM
- Auto-creates schema on first run
- Persists to `.stone/stone.db`

## 📦 Bundle Size Comparison

| Metric | Electron (TypeScript) | Tauri (Rust) | Savings |
|--------|----------------------|--------------|---------|
| **Debug Build** | ~170-200 MB | **27 MB** | **~85%** |
| **Release Build** | ~170-200 MB | **~15-20 MB** | **~90%** |

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Run in development mode (app will open automatically)
npm run tauri dev

# Build for production
npm run tauri build

# The app bundle will be at:
# src-tauri/target/debug/bundle/macos/stone-tauri.app (27 MB)
```

## 🏗️ Architecture

```
Frontend (React)
      ↓ Tauri IPC
┌─────────────────────────────────┐
│  Adapter IN (Tauri Commands)    │
├─────────────────────────────────┤
│  Application (Use Cases)        │
├─────────────────────────────────┤
│  Domain (Entities + Ports)      │
├─────────────────────────────────┤
│  Adapter OUT (Diesel Repository)│
└─────────────────────────────────┘
          ↓
   SQLite Database
   (.stone/stone.db)
```

## 📝 Available Commands

### Note CRUD
- `create_note` - Create a new note with validation
- `get_note_by_id` - Get note by ID
- `get_all_notes` - List all notes with filters
- `update_note` - Update note properties
- `delete_note` - Soft delete note
- `permanently_delete_note` - Hard delete
- `restore_note` - Restore deleted note

### Organization
- `toggle_favorite` - Toggle favorite status
- `toggle_pin` - Toggle pin status
- `archive_note` / `unarchive_note`
- `move_to_notebook` - Move to notebook

### Queries
- `get_recent_notes` - Recently updated notes
- `get_favorites` - Favorite notes only
- `get_archived` - Archived notes
- `get_trash` - Deleted notes

## 🔧 Tech Stack

### Backend (Rust)
- **Tauri 2.9** - Native app framework
- **Diesel 2.3** - ORM + migrations
- **Tokio 1.x** - Async runtime
- **Chrono** - DateTime handling
- **Nanoid** - ID generation
- **Thiserror** - Error handling

### Frontend (TypeScript)
- **React 18** - UI framework
- **TypeScript 5.x** - Type safety
- **Vite 6.x** - Build tool
- **Tauri API** - IPC communication

## 📁 Project Structure

```
src-tauri/src/
├── domain/              # Pure business logic (NO dependencies)
│   ├── entities/        # Note entity with validation
│   ├── ports/
│   │   ├── inbound/     # Use case interfaces (IN ports)
│   │   └── out/         # Repository interfaces (OUT ports)
│   ├── errors/          # Domain errors
│   └── services/        # Pure business logic services
│
├── application/         # Use cases (orchestration)
│   └── usecases/
│       └── note_usecases_impl.rs
│
├── adapters/
│   ├── inbound/         # Tauri commands (entry points)
│   │   └── commands/
│   │       └── note_commands.rs
│   └── out/
│       └── persistence/ # Diesel repository implementation
│           └── diesel_note_repository.rs
│
├── infrastructure/
│   ├── di/              # Dependency injection container
│   │   └── container.rs
│   └── database/        # DB setup & migrations
│       └── mod.rs
│
└── shared/
    └── database/        # Diesel schema
        └── schema.rs
```

## 🧪 Testing

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build release (optimized)
cargo build --release

# Check for issues
cargo clippy
```

## 📊 Key Benefits

1. **90% Smaller**: 27 MB debug, ~15-20 MB release vs 170-200 MB Electron
2. **Same Architecture**: Hexagonal design perfectly preserved
3. **Type Safe**: Compile-time guarantees (Rust + TypeScript)
4. **Fast**: Zero-cost abstractions, no garbage collection
5. **Secure**: Rust safety guarantees + Tauri security model
6. **Native**: System webview, no Chromium bundled
7. **Cross-Platform**: macOS, Windows, Linux from same codebase

## 🔄 TypeScript → Rust Mapping

| TypeScript/Electron | Rust/Tauri | Notes |
|---------------------|------------|-------|
| `interface INoteRepository` | `trait NoteRepository` | Same concept |
| `class NoteRepository implements INoteRepository` | `impl NoteRepository for DieselNoteRepository` | Implementation |
| `async function createNote()` | `async fn create_note()` | Async/await works similarly |
| `Promise<Note>` | `DomainResult<Note>` | Result type for errors |
| `throw new Error()` | `Err(DomainError::...)` | Explicit error handling |
| `constructor(repo: IRepo)` | `Arc<dyn Repository>` | DI via Arc smart pointers |
| `ipcMain.handle()` | `#[tauri::command]` | IPC handler |
| `ipcRenderer.invoke()` | `invoke()` from `@tauri-apps/api` | IPC client |

## 📚 Learn More

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Detailed architecture guide with examples
- [Tauri Docs](https://tauri.app/) - Tauri framework documentation
- [Diesel ORM](https://diesel.rs/) - Database ORM
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/) - Architecture pattern

## 🎓 Next Steps - Adding Features

To add a new feature (e.g., Notebooks):

1. **Create Entity** (`domain/entities/notebook.rs`)
   ```rust
   pub struct Notebook {
       pub id: String,
       pub name: String,
       // ...
   }
   ```

2. **Define OUT Port** (`domain/ports/out/notebook_repository.rs`)
   ```rust
   #[async_trait]
   pub trait NotebookRepository: Send + Sync {
       async fn find_by_id(&self, id: &str) -> DomainResult<Option<Notebook>>;
   }
   ```

3. **Define IN Port** (`domain/ports/inbound/notebook_usecases.rs`)
   ```rust
   #[async_trait]
   pub trait NotebookUseCases: Send + Sync {
       async fn create_notebook(&self, input: CreateNotebookInput) -> DomainResult<Notebook>;
   }
   ```

4. **Implement Use Case** (`application/usecases/notebook_usecases_impl.rs`)
   ```rust
   impl NotebookUseCases for NotebookUseCasesImpl { /* ... */ }
   ```

5. **Implement Repository** (`adapters/out/persistence/diesel_notebook_repository.rs`)
   ```rust
   impl NotebookRepository for DieselNotebookRepository { /* ... */ }
   ```

6. **Add Tauri Commands** (`adapters/inbound/commands/notebook_commands.rs`)
   ```rust
   #[tauri::command]
   pub async fn create_notebook(/* ... */) -> Result<Notebook, String> { /* ... */ }
   ```

7. **Wire DI Container** (`infrastructure/di/container.rs`)
   ```rust
   let notebook_repo = Arc::new(DieselNotebookRepository::new(pool));
   let notebook_usecases = Arc::new(NotebookUseCasesImpl::new(notebook_repo));
   ```

8. **Register Commands** (`lib.rs`)
   ```rust
   .invoke_handler(tauri::generate_handler![
       create_notebook,
       // ...
   ])
   ```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed examples and best practices.

## 🐛 Debugging

```bash
# View database
sqlite3 .stone/stone.db
> SELECT * FROM notes;

# Enable Rust logging
RUST_LOG=debug npm run tauri dev

# Check app logs
# macOS: ~/Library/Logs/stone-tauri/
```

## 📝 Database Schema

```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    notebook_id TEXT,
    workspace_id TEXT,
    file_path TEXT,
    is_favorite INTEGER DEFAULT 0,
    is_pinned INTEGER DEFAULT 0,
    is_archived INTEGER DEFAULT 0,
    is_deleted INTEGER DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

**Built with 🦀 Rust + Tauri** • [Original TypeScript/Electron Version](../stone)

*Successfully compiled and running! Check the dev server at http://localhost:1420/*
