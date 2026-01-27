# Contributing to Stone

Thank you for your interest in contributing to Stone! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, constructive, and professional in all interactions. We're building software together and maintaining a welcoming environment for everyone.

## Getting Started

### Prerequisites

- **Rust** 1.70 or later ([Install](https://rustup.rs/))
- **Node.js** 20 or later
- **pnpm** package manager ([Install](https://pnpm.io/installation))
- **Git** for version control

### Setting Up Development Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/stone-tauri.git
cd stone-tauri

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Run tests
cargo test
```

## Architecture

Stone uses **Hexagonal Architecture** (Ports and Adapters). Please read [ARCHITECTURE.md](./ARCHITECTURE.md) before making changes.

**Key principles:**
- Domain layer has no external dependencies
- Application layer orchestrates use cases
- Adapters connect to external systems
- Dependencies point inward

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

**Branch naming:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions

### 2. Make Changes

Follow the architectural guidelines when adding features:

1. Define domain entities in `src-tauri/src/domain/entities/`
2. Create ports (interfaces) in `src-tauri/src/domain/ports/`
3. Implement use cases in `src-tauri/src/application/usecases/`
4. Implement adapters in `src-tauri/src/adapters/`
5. Wire dependencies in `src-tauri/src/infrastructure/container.rs`
6. Build UI in `src/components/`

### 3. Write Tests

**Backend (Rust):**
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

**Frontend (TypeScript):**
```bash
# Run tests (when implemented)
pnpm test
```

### 4. Code Quality

**Before committing:**

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Type check TypeScript
pnpm typecheck

# Check compilation
cargo check
```

### 5. Commit Changes

Use descriptive commit messages following [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions
- `chore:` - Build process or tooling changes

**Examples:**
```
feat(editor): add markdown table support

fix(quick-capture): resolve window positioning on multi-monitor setup

docs(architecture): add hexagonal architecture diagram

refactor(repository): extract common database operations
```

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title describing the change
- Description of what changed and why
- Any related issues (e.g., "Closes #123")
- Screenshots for UI changes

## Code Style

### Rust

Follow Rust best practices and idioms:

```rust
// Good: Clear, descriptive names
pub async fn create_note(&self, title: String) -> DomainResult<Note> {
    // Validation first
    if title.trim().is_empty() {
        return Err(DomainError::ValidationError("Title required".into()));
    }

    // Business logic
    let note = Note::new(title)?;
    self.repository.save(&note).await?;

    Ok(note)
}

// Bad: Unclear names, no validation
pub async fn create(&self, t: String) -> Result<Note, Box<dyn Error>> {
    let n = Note::new(t)?;
    self.repo.save(&n).await?;
    Ok(n)
}
```

**Rust conventions:**
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Use `SCREAMING_SNAKE_CASE` for constants
- Prefer explicit error types over `Box<dyn Error>`
- Use `async/await` for async code
- Document public APIs with `///` doc comments

### TypeScript

Follow TypeScript best practices:

```typescript
// Good: Type-safe, clear naming
interface CreateNoteInput {
  title: string;
  workspaceId?: string;
}

async function createNote(input: CreateNoteInput): Promise<Note> {
  const result = await noteAPI.create(input);
  if (!result.success) {
    throw new Error(result.error?.message || 'Failed to create note');
  }
  return result.data!;
}

// Bad: Any types, unclear
async function create(data: any): Promise<any> {
  const result = await api.create(data);
  return result.data;
}
```

**TypeScript conventions:**
- Use `camelCase` for variables and functions
- Use `PascalCase` for types and interfaces
- Avoid `any` - use proper types or `unknown`
- Use optional chaining `?.` and nullish coalescing `??`
- Prefer `interface` over `type` for object shapes

### React

Follow React best practices:

```typescript
// Good: Hooks, clear component structure
export function NoteEditor({ noteId }: { noteId: string }) {
  const [note, setNote] = useState<Note | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadNote(noteId);
  }, [noteId]);

  async function loadNote(id: string) {
    setLoading(true);
    const result = await noteAPI.get(id);
    if (result.success) {
      setNote(result.data);
    }
    setLoading(false);
  }

  if (loading) return <Spinner />;
  if (!note) return <EmptyState />;

  return <Editor content={note.content} />;
}
```

## Testing Guidelines

### Unit Tests (Domain Layer)

Test business logic in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_title_validation() {
        // Arrange
        let empty_title = "";

        // Act
        let result = Note::new(empty_title);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(DomainError::ValidationError(_))));
    }
}
```

### Integration Tests (Use Cases)

Test use cases with mock dependencies:

```rust
#[tokio::test]
async fn test_create_note_use_case() {
    // Arrange
    let mock_repo = Arc::new(MockNoteRepository::new());
    let use_case = NoteUseCasesImpl::new(mock_repo);

    // Act
    let note = use_case.create_note("Test Note".into()).await.unwrap();

    // Assert
    assert_eq!(note.title, "Test Note");
}
```

### Component Tests (Frontend)

Test React components:

```typescript
test('displays note title', () => {
  render(<NoteCard note={mockNote} />);
  expect(screen.getByText(mockNote.title)).toBeInTheDocument();
});
```

## Documentation

### Code Documentation

**Rust:**
```rust
/// Creates a new note with the given title.
///
/// # Arguments
///
/// * `title` - The title of the note (cannot be empty)
///
/// # Returns
///
/// * `Ok(Note)` - Successfully created note
/// * `Err(DomainError)` - Validation error if title is empty
///
/// # Examples
///
/// ```
/// let note = Note::new("My Note")?;
/// assert_eq!(note.title, "My Note");
/// ```
pub fn new(title: impl Into<String>) -> DomainResult<Self> {
    // Implementation
}
```

**TypeScript:**
```typescript
/**
 * Creates a new note via the backend API.
 *
 * @param input - Note creation parameters
 * @returns Promise resolving to the created note
 * @throws Error if creation fails
 */
async function createNote(input: CreateNoteInput): Promise<Note> {
  // Implementation
}
```

### User-Facing Documentation

Update relevant documentation when adding features:
- README.md - For major features
- docs/ - For detailed guides
- Code comments - For complex logic

## Pull Request Process

### Before Submitting

- [ ] Code compiles without warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`, `pnpm format`)
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] Commit messages follow convention
- [ ] Branch is up to date with main

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
How was this tested?

## Screenshots (if applicable)
Add screenshots for UI changes

## Related Issues
Closes #123
```

### Review Process

1. Automated checks must pass (CI)
2. Code review by maintainer
3. Address feedback
4. Approval and merge

## Architecture Guidelines

### Adding a New Entity

1. **Domain Entity** (`domain/entities/feature.rs`)
```rust
pub struct Feature {
    pub id: String,
    pub name: String,
}

impl Feature {
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        // Validation
    }
}
```

2. **Repository Port** (`domain/ports/outbound/feature_repository.rs`)
```rust
#[async_trait]
pub trait FeatureRepository: Send + Sync {
    async fn save(&self, feature: &Feature) -> DomainResult<()>;
}
```

3. **Use Case Port** (`domain/ports/inbound/feature_usecases.rs`)
```rust
#[async_trait]
pub trait FeatureUseCases: Send + Sync {
    async fn create(&self, name: String) -> DomainResult<Feature>;
}
```

4. **Implementation** (`application/usecases/feature_usecases.rs`)
5. **Repository** (`adapters/outbound/persistence/feature_repository.rs`)
6. **Commands** (`adapters/inbound/feature_commands.rs`)
7. **Wire DI** (`infrastructure/container.rs`)

See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete details.

## Common Issues

### Compilation Errors

**Problem:** Diesel schema out of sync
```
error: no field `new_column` on table `notes`
```

**Solution:**
```bash
diesel migration run
diesel print-schema > src-tauri/src/shared/database/schema.rs
```

### Runtime Errors

**Problem:** Database locked
```
Error: database is locked
```

**Solution:** Ensure file watcher isn't conflicting, check for hanging connections.

## Getting Help

- Read [ARCHITECTURE.md](./ARCHITECTURE.md)
- Check existing issues on GitHub
- Ask questions in discussions
- Reach out to maintainers

## Recognition

Contributors will be recognized in:
- GitHub contributors page
- Release notes for significant contributions
- Project documentation

Thank you for contributing to Stone!
