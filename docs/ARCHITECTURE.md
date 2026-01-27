# Architecture

Stone is built using **Hexagonal Architecture** (also known as Ports and Adapters), providing a clean separation of concerns and making the codebase maintainable, testable, and adaptable to change.

## Overview

```
┌─────────────────────────────────────┐
│   Frontend (React + TypeScript)    │
│   • TipTap Editor                   │
│   • Zustand State Management        │
│   • Radix UI Components             │
└──────────────┬──────────────────────┘
               │ Tauri IPC
┌──────────────▼──────────────────────┐
│     Adapters IN (Tauri Commands)    │
├─────────────────────────────────────┤
│   Application Layer (Use Cases)     │
│   • Note, Workspace, Tag, Topic     │
│   • Search, Task, Graph, Git        │
├─────────────────────────────────────┤
│      Domain Layer (Entities)        │
│   • Pure business logic             │
│   • No framework dependencies       │
├─────────────────────────────────────┤
│    Adapters OUT (Implementations)   │
│   • Diesel Repository (SQLite)      │
│   • File Storage (Tokio)            │
│   • Event Publisher (Broadcast)     │
│   • File Watcher (Notify)           │
└──────────────┬──────────────────────┘
               │
        ┌──────▼─────┬────────────┐
        │            │            │
    SQLite      Markdown       Events
  (Metadata)    (Content)    (Real-time)
```

## Layer Responsibilities

### Domain Layer (`src-tauri/src/domain`)

The innermost layer containing pure business logic with **no external dependencies**.

**Structure:**
```
domain/
├── entities/           # Business entities (Note, Workspace, Tag, etc.)
├── value_objects/      # Immutable values (HexColor, FilePath, etc.)
├── services/           # Domain services (TaskExtractor, LinkExtractor)
└── ports/              # Interface definitions (traits)
    ├── inbound/        # Use case interfaces
    └── outbound/       # Repository and service interfaces
```

**Rules:**
- No framework dependencies (no Tauri, Diesel, etc.)
- Pure Rust with minimal external crates
- All business rules live here
- Defines interfaces (ports) for external services
- Should be testable without any infrastructure

**Example Entity:**
```rust
pub struct Note {
    pub id: String,
    pub title: String,
    pub workspace_id: Option<String>,
    pub is_favorite: bool,
    // ... other fields
}

impl Note {
    pub fn new(title: impl Into<String>) -> DomainResult<Self> {
        // Business rule: Title cannot be empty
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Note title cannot be empty".to_string()
            ));
        }
        // ... create note
    }
}
```

### Application Layer (`src-tauri/src/application`)

Orchestrates business logic by implementing use cases. This layer coordinates between domain entities and external services.

**Structure:**
```
application/
└── usecases/          # Use case implementations
    ├── note_usecases.rs
    ├── workspace_usecases.rs
    ├── tag_usecases.rs
    └── ...
```

**Rules:**
- Implements inbound ports (use case traits from domain)
- Depends on domain layer only
- Coordinates multiple domain entities
- No direct infrastructure access (uses outbound ports)
- Transaction boundaries defined here

**Example Use Case:**
```rust
pub struct NoteUseCasesImpl {
    note_repository: Arc<dyn NoteRepository>,
    file_storage: Arc<dyn FileStorage>,
    markdown_processor: Arc<dyn MarkdownProcessor>,
}

impl NoteUseCases for NoteUseCasesImpl {
    async fn create_note(&self, input: CreateNoteInput) -> DomainResult<Note> {
        // 1. Create domain entity
        let note = Note::new(input.title)?;

        // 2. Save to database
        self.note_repository.save(&note).await?;

        // 3. Create file
        let content = self.markdown_processor.generate_template(&note)?;
        self.file_storage.write(&note.file_path, &content).await?;

        Ok(note)
    }
}
```

### Adapters Layer (`src-tauri/src/adapters`)

Implements the interfaces defined by the domain layer, connecting to external systems.

#### Inbound Adapters (`adapters/inbound`)

Entry points into the application. For Stone, these are Tauri commands.

**Structure:**
```
adapters/inbound/
├── app_state.rs           # Container for all use cases
├── note_commands.rs       # Note-related commands
├── workspace_commands.rs  # Workspace-related commands
└── ...
```

**Example Command:**
```rust
#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    input: CreateNoteInput,
) -> Result<Note, String> {
    state.note_usecases
        .create_note(input)
        .await
        .map_err(|e| e.to_string())
}
```

#### Outbound Adapters (`adapters/outbound`)

Implementations of domain ports for external services.

**Structure:**
```
adapters/outbound/
├── persistence/          # Database implementations
│   ├── note_repository.rs
│   ├── workspace_repository.rs
│   └── mappers/         # Domain ↔ Database mapping
├── services/            # Service implementations
│   ├── file_storage_impl.rs
│   ├── markdown_service.rs
│   ├── git_service_impl.rs
│   └── event_publisher_impl.rs
└── storage/             # File system operations
```

**Example Repository:**
```rust
pub struct DieselNoteRepository {
    pool: Arc<DbPool>,
}

impl NoteRepository for DieselNoteRepository {
    async fn save(&self, note: &Note) -> DomainResult<()> {
        let mut conn = self.pool.get()?;
        let db_note = NoteMapper::to_database(note);

        diesel::insert_into(notes::table)
            .values(&db_note)
            .on_conflict(notes::id)
            .do_update()
            .set(&db_note)
            .execute(&mut conn)?;

        Ok(())
    }
}
```

### Infrastructure Layer (`src-tauri/src/infrastructure`)

Cross-cutting concerns and application setup.

**Structure:**
```
infrastructure/
├── container.rs           # Dependency injection
├── database/              # Database setup & migrations
│   └── manager.rs
└── config/                # Configuration loading
    ├── app_config.rs
    └── database_config.rs
```

**Responsibilities:**
- Dependency injection container
- Database initialization and migrations
- Configuration management
- Application bootstrapping

### Shared Layer (`src-tauri/src/shared`)

Code shared across layers (use sparingly).

**Structure:**
```
shared/
└── database/
    └── schema.rs         # Diesel schema definitions
```

## Dependency Rules

**Key Principle:** Dependencies point inward.

```
Adapters → Application → Domain
    ↓
Infrastructure
```

**Allowed Dependencies:**
- Domain: No external dependencies
- Application: Domain only
- Adapters: Domain + Application + External libraries
- Infrastructure: All layers

**Forbidden:**
- Domain cannot depend on Application, Adapters, or Infrastructure
- Application cannot depend on Adapters or Infrastructure
- Inner layers cannot know about outer layers

## Data Flow

### Inbound Flow (User Action → Response)

```
1. User Action (Frontend)
   ↓
2. Tauri IPC Call
   ↓
3. Inbound Adapter (Tauri Command)
   ↓
4. Application Layer (Use Case)
   ↓
5. Domain Layer (Entity + Business Logic)
   ↓
6. Outbound Adapter (Repository/Service)
   ↓
7. External System (Database/File System)
```

### Outbound Flow (External Event → Application)

```
1. External Event (File Change)
   ↓
2. Outbound Adapter (File Watcher)
   ↓
3. Event Publisher
   ↓
4. Application Layer (Event Handler)
   ↓
5. Domain Layer (Business Logic)
   ↓
6. Inbound Adapter (Emit to Frontend)
   ↓
7. Frontend Update
```

## Testing Strategy

### Domain Layer Tests
Pure unit tests, no mocking required.

```rust
#[test]
fn test_note_creation() {
    let note = Note::new("Test Note").unwrap();
    assert_eq!(note.title, "Test Note");
}

#[test]
fn test_empty_title_validation() {
    let result = Note::new("");
    assert!(result.is_err());
}
```

### Application Layer Tests
Mock outbound ports using trait objects.

```rust
#[tokio::test]
async fn test_create_note_use_case() {
    let mock_repo = Arc::new(MockNoteRepository::new());
    let mock_storage = Arc::new(MockFileStorage::new());

    let use_case = NoteUseCasesImpl::new(mock_repo, mock_storage);

    let note = use_case.create_note(input).await.unwrap();
    assert_eq!(note.title, "Test");
}
```

### Adapter Tests
Integration tests with real implementations.

```rust
#[tokio::test]
async fn test_diesel_repository() {
    let pool = create_test_pool();
    let repo = DieselNoteRepository::new(pool);

    let note = Note::new("Test").unwrap();
    repo.save(&note).await.unwrap();

    let loaded = repo.find_by_id(&note.id).await.unwrap();
    assert_eq!(loaded.title, note.title);
}
```

## Adding a New Feature

Follow these steps to maintain architectural integrity:

### 1. Define Domain Entity
```rust
// src-tauri/src/domain/entities/feature.rs
pub struct Feature {
    pub id: String,
    pub name: String,
}

impl Feature {
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        // Validation logic
    }
}
```

### 2. Define Outbound Port
```rust
// src-tauri/src/domain/ports/outbound/feature_repository.rs
#[async_trait]
pub trait FeatureRepository: Send + Sync {
    async fn save(&self, feature: &Feature) -> DomainResult<()>;
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Feature>>;
}
```

### 3. Define Inbound Port
```rust
// src-tauri/src/domain/ports/inbound/feature_usecases.rs
#[async_trait]
pub trait FeatureUseCases: Send + Sync {
    async fn create_feature(&self, name: String) -> DomainResult<Feature>;
}
```

### 4. Implement Use Case
```rust
// src-tauri/src/application/usecases/feature_usecases.rs
pub struct FeatureUseCasesImpl {
    repository: Arc<dyn FeatureRepository>,
}

impl FeatureUseCases for FeatureUseCasesImpl {
    async fn create_feature(&self, name: String) -> DomainResult<Feature> {
        let feature = Feature::new(name)?;
        self.repository.save(&feature).await?;
        Ok(feature)
    }
}
```

### 5. Implement Repository
```rust
// src-tauri/src/adapters/outbound/persistence/feature_repository.rs
pub struct DieselFeatureRepository {
    pool: Arc<DbPool>,
}

impl FeatureRepository for DieselFeatureRepository {
    async fn save(&self, feature: &Feature) -> DomainResult<()> {
        // Diesel implementation
    }
}
```

### 6. Add Tauri Commands
```rust
// src-tauri/src/adapters/inbound/feature_commands.rs
#[tauri::command]
pub async fn create_feature(
    state: State<'_, AppState>,
    name: String,
) -> Result<Feature, String> {
    state.feature_usecases
        .create_feature(name)
        .await
        .map_err(|e| e.to_string())
}
```

### 7. Wire Dependencies
```rust
// src-tauri/src/infrastructure/container.rs
let feature_repo = Arc::new(DieselFeatureRepository::new(pool.clone()));
let feature_usecases = Arc::new(FeatureUseCasesImpl::new(feature_repo));

app_state.feature_usecases = feature_usecases;
```

### 8. Register Commands
```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    feature_commands::create_feature,
    // ...
])
```

## Best Practices

### Do's
- Keep domain layer pure (no external dependencies)
- Use dependency injection via Arc<dyn Trait>
- Define clear port interfaces
- Write tests at each layer
- Keep entities focused (single responsibility)
- Use value objects for complex values
- Handle errors explicitly with Result types

### Don'ts
- Don't put business logic in adapters
- Don't access database directly from application layer
- Don't use concrete types across layer boundaries
- Don't mix UI concerns with business logic
- Don't bypass the architecture for "quick fixes"
- Don't use global state or singletons
- Don't return implementation types from ports

## Common Patterns

### Repository Pattern
Abstracts data access behind an interface.

### Use Case Pattern
Each use case is a separate struct with focused responsibility.

### Dependency Injection
Constructor injection using Arc smart pointers.

### Event Publishing
Decouple components using domain events.

### Error Handling
Custom error types per layer, converted at boundaries.

## Further Reading

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design](https://martinfowler.com/bliki/DomainDrivenDesign.html)
- [Ports and Adapters](https://herbertograca.com/2017/09/14/ports-adapters-architecture/)
