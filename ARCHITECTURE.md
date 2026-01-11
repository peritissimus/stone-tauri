# Stone Tauri - Architecture Guide

## 🏗️ Hexagonal Architecture (Ports & Adapters)

This is a **Tauri + Rust** implementation following the same hexagonal architecture as the original TypeScript/Electron version.

```
┌─────────────────────────────────────────────────┐
│            FRONTEND (React + TypeScript)        │
│              (Same as before)                   │
├─────────────────────────────────────────────────┤
│                 TAURI IPC LAYER                 │
│          (Replaces Electron IPC)                │
├─────────────────────────────────────────────────┤
│              BACKEND (Rust)                     │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │         ADAPTERS/INBOUND                  │ │
│  │    (Tauri Commands - Entry Points)        │ │
│  └─────────────┬─────────────────────────────┘ │
│                ▼                                │
│  ┌───────────────────────────────────────────┐ │
│  │         APPLICATION LAYER                 │ │
│  │         (Use Cases - NoteUseCasesImpl)    │ │
│  └─────────────┬─────────────────────────────┘ │
│                ▼                                │
│  ┌───────────────────────────────────────────┐ │
│  │           DOMAIN LAYER                    │ │
│  │  (Entities, Ports, Business Logic)        │ │
│  └─────┬─────────────────────┬───────────────┘ │
│        │                     │                  │
│        ▼                     ▼                  │
│  ┌─────────────┐    ┌──────────────────┐       │
│  │ ADAPTERS/   │    │  INFRASTRUCTURE  │       │
│  │ OUT         │    │  (DI Container,  │       │
│  │ (Diesel     │    │   DB Setup)      │       │
│  │  Repository)│    └──────────────────┘       │
│  └──────┬──────┘                                │
│         │                                       │
│         ▼                                       │
│  ┌─────────────────┐                            │
│  │ SQLite Database │                            │
│  └─────────────────┘                            │
└─────────────────────────────────────────────────┘
```

## 📁 Directory Structure

```
src-tauri/src/
├── domain/                    # PURE BUSINESS LOGIC (zero dependencies)
│   ├── entities/              # Business objects (Note)
│   │   └── note.rs
│   ├── errors/                # Domain errors
│   │   └── mod.rs
│   ├── ports/                 # Contracts/interfaces
│   │   ├── inbound/           # What app CAN DO (use cases)
│   │   │   └── note_usecases.rs
│   │   └── out/               # What app NEEDS (repositories)
│   │       └── note_repository.rs
│   └── services/              # Pure business calculations
│
├── application/               # USE CASES (orchestration)
│   ├── usecases/
│   │   └── note_usecases_impl.rs
│   └── dto/
│
├── adapters/                  # EXTERNAL INTEGRATIONS
│   ├── inbound/               # Entry points (Tauri commands)
│   │   └── commands/
│   │       └── note_commands.rs
│   └── out/                   # External implementations
│       └── persistence/
│           └── diesel_note_repository.rs
│
├── infrastructure/            # WIRING & SETUP
│   ├── di/                    # Dependency injection
│   │   └── container.rs
│   └── database/              # Database setup
│       └── mod.rs
│
└── shared/                    # SHARED UTILITIES
    └── database/
        └── schema.rs          # Diesel schema
```

## 🔄 TypeScript → Rust Mapping

### Dependencies

| TypeScript Package | Rust Crate | Purpose |
|-------------------|------------|---------|
| `electron` | `tauri` | Desktop app framework |
| `drizzle-orm` | `diesel` | ORM |
| `@libsql/client` | `rusqlite` (via diesel) | SQLite driver |
| `simple-git` | `git2` | Git operations |
| `marked` | `pulldown-cmark` | Markdown → HTML |
| `chokidar` | `notify` | File watching |
| `nanoid` | `nanoid` | ID generation |
| `zod` | `serde` + traits | Validation/serialization |

### Code Structure

| TypeScript | Rust |
|-----------|------|
| `interface INoteRepository` | `trait NoteRepository` |
| `class NoteRepository implements INoteRepository` | `impl NoteRepository for DieselNoteRepository` |
| `async function createNote()` | `async fn create_note()` |
| `Promise<Note>` | `DomainResult<Note>` |
| `throw new Error()` | `Err(DomainError::...)` |
| Constructor DI: `constructor(private repo: IRepo)` | `pub fn new(repo: Arc<dyn Repository>)` |

## 🚀 Available Commands

### 15+ Tauri Commands (IPC)

```rust
// Note CRUD
create_note(input: CreateNoteInput) -> Note
get_note_by_id(id: String) -> Option<Note>
get_all_notes(query: NoteQuery) -> Vec<Note>
update_note(input: UpdateNoteInput) -> Note
delete_note(id: String) -> ()

// Soft delete operations
permanently_delete_note(id: String) -> ()
restore_note(id: String) -> ()

// Toggle operations
toggle_favorite(id: String) -> Note
toggle_pin(id: String) -> Note

// Archive operations
archive_note(id: String) -> ()
unarchive_note(id: String) -> ()

// Organization
move_to_notebook(note_id: String, notebook_id: Option<String>) -> ()

// Queries
get_recent_notes(limit: i64, workspace_id: Option<String>) -> Vec<Note>
get_favorites(workspace_id: Option<String>) -> Vec<Note>
get_archived(workspace_id: Option<String>) -> Vec<Note>
get_trash(workspace_id: Option<String>) -> Vec<Note>
```

## 📊 Bundle Size Comparison

| Metric | Electron | Tauri | Savings |
|--------|----------|-------|---------|
| **Runtime** | Chromium (~120 MB) | System WebView (0 MB) | **120 MB** |
| **Node modules** | ~50-80 MB | Compiled binary (~5-10 MB) | **45-75 MB** |
| **Total** | ~170-200 MB | ~15-25 MB | **~85-90%** |

## 🎯 Key Benefits

1. **Same Architecture**: Hexagonal design preserved
2. **90% Smaller**: ~15-25 MB vs 170-200 MB
3. **Type Safety**: Compile-time guarantees
4. **Performance**: Zero-cost abstractions, no GC
5. **Memory**: Lower footprint
6. **Security**: Rust's safety guarantees

## 🔧 How to Run

```bash
# Development
npm run tauri dev

# Build
npm run tauri build

# Database location
# Dev: .stone/stone.db
# Prod: Platform-specific app data directory
```

## 📝 Adding New Features

### 1. Add Entity (Domain)
```rust
// src/domain/entities/notebook.rs
pub struct Notebook {
    pub id: String,
    pub name: String,
    // ... fields
}
```

### 2. Define Ports (Domain)
```rust
// src/domain/ports/out/notebook_repository.rs
#[async_trait]
pub trait NotebookRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Notebook>>;
    // ... methods
}
```

### 3. Implement Use Case (Application)
```rust
// src/application/usecases/notebook_usecases_impl.rs
pub struct NotebookUseCasesImpl {
    repository: Arc<dyn NotebookRepository>,
}

#[async_trait]
impl NotebookUseCases for NotebookUseCasesImpl {
    async fn create_notebook(&self, input: CreateNotebookInput) -> DomainResult<Notebook> {
        // orchestrate
    }
}
```

### 4. Implement Repository (Adapter OUT)
```rust
// src/adapters/out/persistence/diesel_notebook_repository.rs
pub struct DieselNotebookRepository { /* ... */ }

#[async_trait]
impl NotebookRepository for DieselNotebookRepository {
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Notebook>> {
        // Diesel queries
    }
}
```

### 5. Add Tauri Commands (Adapter IN)
```rust
// src/adapters/inbound/commands/notebook_commands.rs
#[tauri::command]
pub async fn create_notebook(
    input: CreateNotebookInput,
    state: State<'_, AppState>,
) -> Result<Notebook, String> {
    state.notebook_usecases.create_notebook(input).await
        .map_err(|e| e.to_string())
}
```

### 6. Wire in DI Container (Infrastructure)
```rust
// src/infrastructure/di/container.rs
let notebook_repo = Arc::new(DieselNotebookRepository::new(pool));
let notebook_usecases = Arc::new(NotebookUseCasesImpl::new(notebook_repo));
```

### 7. Register in lib.rs
```rust
.invoke_handler(tauri::generate_handler![
    create_notebook,
    // ...
])
```

## 🧪 Testing

```bash
# Unit tests
cargo test

# Check for errors
cargo check

# Run with logging
RUST_LOG=debug cargo run
```

## 🎓 Learn More

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Tauri Docs](https://tauri.app/)
- [Diesel ORM](https://diesel.rs/)
- [Rust Book](https://doc.rust-lang.org/book/)
