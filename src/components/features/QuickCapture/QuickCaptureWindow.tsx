/**
 * QuickCaptureWindow - Floating window for quick journal capture
 *
 * Optimized for speed: closes immediately on submit, save happens in background
 */

import React, { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useQuickCaptureAPI } from "@/hooks/useQuickCaptureAPI";
import { quickCaptureAPI } from "@/api/quickCaptureAPI";

// Log to backend process
const backendLog = (message: string, level?: "info" | "warn" | "error" | "debug") => {
  invoke("log_from_frontend", { message, level }).catch(() => {});
};

const DRAFT_KEY = "quick-capture-draft";

export function QuickCaptureWindow() {
  const { appendToJournal } = useQuickCaptureAPI();
  const [text, setText] = useState(() => {
    // Restore draft on mount
    return localStorage.getItem(DRAFT_KEY) || "";
  });
  const [isFocused, setIsFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Track window visibility - synced from backend events (single source of truth)
  const isVisibleRef = useRef(false);

  // Listen for backend state changes - this is the ONLY way visibility changes
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<string>("quick-capture:state-changed", (event) => {
        const newState = event.payload;
        backendLog(`QuickCapture: Backend state changed to ${newState}`);

        if (newState === "Visible") {
          isVisibleRef.current = true;
          setIsFocused(true);
          // Focus the textarea when window becomes visible
          setTimeout(() => textareaRef.current?.focus(), 50);
        } else if (newState === "Hidden") {
          isVisibleRef.current = false;
          setIsFocused(false);
        }
      });

      return unlisten;
    };

    const unlistenPromise = setupListener();

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const closeWindow = async () => {
    // Only close if we're visible (tracked from backend events)
    if (!isVisibleRef.current) {
      backendLog("QuickCapture: Not visible, ignoring close");
      return;
    }

    // Mark as not visible immediately to prevent duplicate calls
    isVisibleRef.current = false;
    backendLog("QuickCapture: Attempting to hide window");

    try {
      const response = await quickCaptureAPI.hide();
      if (response.success && response.data?.success) {
        backendLog(`QuickCapture: Window hidden, state: ${response.data.state}`);
      } else {
        backendLog(`QuickCapture: Failed to hide: ${response.data?.error || response.error}`, "error");
      }
    } catch (err) {
      backendLog(`QuickCapture: Hide error: ${err}`, "error");
    }
  };

  // Track window focus for UI only (blue border indicator)
  // Visibility is tracked separately via backend events
  useEffect(() => {
    const handleWindowFocus = () => {
      backendLog("QuickCapture: Window focus event");
      // Only update UI if we're actually visible
      if (isVisibleRef.current) {
        setIsFocused(true);
        textareaRef.current?.focus();
      }
    };

    const handleWindowBlur = () => {
      backendLog("QuickCapture: Window blur event");
      setIsFocused(false);
    };

    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("blur", handleWindowBlur);

    // Check initial state from backend
    const checkInitialState = async () => {
      try {
        const response = await quickCaptureAPI.getState();
        const backendState = response.data;
        backendLog(`QuickCapture: Initial backend state is ${backendState}`);
        if (backendState === "Visible") {
          isVisibleRef.current = true;
          setIsFocused(document.hasFocus());
          if (document.hasFocus()) {
            textareaRef.current?.focus();
          }
        }
      } catch {
        backendLog("QuickCapture: Could not get initial state", "warn");
      }
    };
    checkInitialState();

    return () => {
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("blur", handleWindowBlur);
    };
  }, []);

  // Single unified keyboard handler with capture phase for reliability
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape to close (no text submission)
      if (e.key === "Escape") {
        backendLog(`QuickCapture: Escape key pressed (isVisible=${isVisibleRef.current})`);
        e.preventDefault();
        e.stopPropagation();
        void closeWindow();
        return;
      }

      // Cmd/Ctrl+Enter to save and close
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        backendLog("QuickCapture: Cmd/Ctrl+Enter pressed, saving");
        e.preventDefault();
        e.stopPropagation();

        const trimmedText = text.trim();
        if (trimmedText) {
          // Clear the text state first
          setText("");
          // Clear draft from localStorage
          localStorage.removeItem(DRAFT_KEY);
          // Close window immediately
          void closeWindow();
          // Save in background
          appendToJournal(trimmedText).catch((err) => {
            backendLog(`QuickCapture: Save failed: ${err}`, "error");
            // If save fails, restore draft so user doesn't lose content
            localStorage.setItem(DRAFT_KEY, trimmedText);
          });
        }
      }
    };

    // Use capture phase to intercept events before they bubble
    window.addEventListener("keydown", handleKeyDown, { capture: true });
    return () =>
      window.removeEventListener("keydown", handleKeyDown, { capture: true });
  }, [text, appendToJournal]);

  // NOTE: Blur event handler removed intentionally
  // It caused accidental window closes during app switching (Cmd+Tab)
  // The window should only close via explicit user actions (Escape, Cmd+Enter)

  // Save draft on text change (debounced naturally by React state)
  useEffect(() => {
    if (text.trim()) {
      localStorage.setItem(DRAFT_KEY, text);
    } else {
      localStorage.removeItem(DRAFT_KEY);
    }
  }, [text]);

  // Detect dark mode
  const isDark =
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;

  // Colors from index.css - using popover colors for floating panel visibility
  // Dark: --popover: 0 0% 10% / 0.85, --popover-foreground: 0 0% 92%
  // Light: --popover: 0 0% 100%, --popover-foreground: 0 0% 12%
  const colors = isDark
    ? {
        background: "hsl(0 0% 10% / 0.5)",
        foreground: "hsl(0 0% 92%)",
        mutedForeground: "hsl(0 0% 65%)",
        border: "hsl(0 0% 50% / 0.2)",
        muted: "hsl(0 0% 30% / 0.5)",
      }
    : {
        background: "hsl(0 0% 100%)",
        foreground: "hsl(0 0% 12%)",
        mutedForeground: "hsl(0 0% 45%)",
        border: "hsl(0 0% 85%)",
        muted: "hsl(0 0% 94%)",
      };

  const containerStyle: React.CSSProperties = {
    height: "100vh",
    width: "100vw",
    padding: 8,
    backgroundColor: colors.background,
    borderRadius: 12,
    border: isFocused ? "2px solid hsl(210 100% 50%)" : "2px solid transparent",
    boxSizing: "border-box",
    transition: "border-color 0.15s ease",
  };

  const textareaStyle: React.CSSProperties = {
    width: "100%",
    height: "100%",
    padding: "12px 16px",
    fontSize: 14,
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
    backgroundColor: "transparent",
    color: colors.foreground,
    borderRadius: 12,
    border: "none",
    outline: "none",
    resize: "none",
  };

  const placeholderColor = colors.mutedForeground;

  return (
    <div style={containerStyle}>
      <style>
        {`
          .quick-capture-textarea::placeholder {
            color: ${placeholderColor};
            opacity: 0.6;
            font-size: 12px;
          }
        `}
      </style>
      <textarea
        ref={textareaRef}
        value={text}
        onChange={(e) => {
          const newText = e.target.value;
          backendLog(`QuickCapture: Typing - text length: ${newText.length}`);
          setText(newText);
        }}
        placeholder="What's on your mind? (Cmd+Enter to save)"
        rows={3}
        style={textareaStyle}
        className="quick-capture-textarea"
      />
    </div>
  );
}
