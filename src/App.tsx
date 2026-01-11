import { useState, useEffect } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

interface Note {
  id: string;
  title: string;
  notebook_id: string | null;
  workspace_id: string | null;
  file_path: string | null;
  is_favorite: boolean;
  is_pinned: boolean;
  is_archived: boolean;
  is_deleted: boolean;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
}

function App() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [newNoteTitle, setNewNoteTitle] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load notes on mount
  useEffect(() => {
    loadNotes();
  }, []);

  async function loadNotes() {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<Note[]>("get_all_notes", {
        query: {
          workspace_id: null,
          notebook_id: null,
          is_favorite: null,
          is_archived: null,
          limit: null,
        },
      });
      setNotes(result);
    } catch (err) {
      setError(`Failed to load notes: ${err}`);
      console.error("Error loading notes:", err);
    } finally {
      setLoading(false);
    }
  }

  async function createNote() {
    if (!newNoteTitle.trim()) {
      setError("Title cannot be empty");
      return;
    }

    try {
      setLoading(true);
      setError(null);
      const note = await invoke<Note>("create_note", {
        input: {
          title: newNoteTitle,
          workspace_id: null,
          notebook_id: null,
          content: null,
        },
      });
      setNotes([note, ...notes]);
      setNewNoteTitle("");
    } catch (err) {
      setError(`Failed to create note: ${err}`);
      console.error("Error creating note:", err);
    } finally {
      setLoading(false);
    }
  }

  async function toggleFavorite(id: string) {
    try {
      const updatedNote = await invoke<Note>("toggle_favorite", { id });
      setNotes(notes.map((n) => (n.id === id ? updatedNote : n)));
    } catch (err) {
      setError(`Failed to toggle favorite: ${err}`);
      console.error("Error toggling favorite:", err);
    }
  }

  async function deleteNote(id: string) {
    try {
      await invoke("delete_note", { id });
      setNotes(notes.filter((n) => n.id !== id));
    } catch (err) {
      setError(`Failed to delete note: ${err}`);
      console.error("Error deleting note:", err);
    }
  }

  return (
    <div className="container">
      <h1>Stone Tauri - Note Taking App</h1>

      <div className="card">
        <h2>🦀 Rust + Tauri + Hexagonal Architecture</h2>
        <p>
          This is a proof-of-concept demonstrating the same hexagonal
          architecture as the TypeScript version, but in Rust with 90% smaller
          bundle size.
        </p>
      </div>

      {error && (
        <div className="error">
          <strong>Error:</strong> {error}
        </div>
      )}

      <div className="card">
        <h3>Create New Note</h3>
        <div className="input-group">
          <input
            type="text"
            value={newNoteTitle}
            onChange={(e) => setNewNoteTitle(e.target.value)}
            placeholder="Enter note title..."
            onKeyPress={(e) => e.key === "Enter" && createNote()}
            disabled={loading}
          />
          <button onClick={createNote} disabled={loading || !newNoteTitle.trim()}>
            {loading ? "Creating..." : "Create Note"}
          </button>
        </div>
      </div>

      <div className="card">
        <div className="notes-header">
          <h3>Notes ({notes.length})</h3>
          <button onClick={loadNotes} disabled={loading}>
            {loading ? "Loading..." : "Refresh"}
          </button>
        </div>

        {notes.length === 0 ? (
          <p className="empty-state">No notes yet. Create one above!</p>
        ) : (
          <div className="notes-list">
            {notes.map((note) => (
              <div key={note.id} className="note-item">
                <div className="note-content">
                  <h4>{note.title}</h4>
                  <small>
                    Created: {new Date(note.created_at).toLocaleString()}
                  </small>
                </div>
                <div className="note-actions">
                  <button
                    onClick={() => toggleFavorite(note.id)}
                    className={note.is_favorite ? "favorite-active" : ""}
                    title="Toggle favorite"
                  >
                    {note.is_favorite ? "⭐" : "☆"}
                  </button>
                  <button
                    onClick={() => deleteNote(note.id)}
                    className="delete-btn"
                    title="Delete note"
                  >
                    🗑️
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="card info">
        <h4>✅ Working Features:</h4>
        <ul>
          <li>Create notes with validation</li>
          <li>List all notes</li>
          <li>Toggle favorite status</li>
          <li>Delete notes (soft delete)</li>
          <li>SQLite database persistence</li>
          <li>Hexagonal architecture (Domain → Application → Adapters)</li>
        </ul>
        <p>
          <strong>Check the console</strong> for database location and logs!
        </p>
      </div>
    </div>
  );
}

export default App;
