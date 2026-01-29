//! Quick Capture Window - NSPanel Implementation
//!
//! This module provides a floating quick capture window using our minimal
//! NSPanel infrastructure with a robust state machine and proper synchronization.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
#[cfg(target_os = "macos")]
use tauri::window::{Effect, EffectsBuilder};

#[cfg(target_os = "macos")]
use crate::infrastructure::nspanel::{
    CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};

const QUICK_CAPTURE_LABEL: &str = "quick-capture";
const QUICK_CAPTURE_URL: &str = "index.html#/quick-capture";
const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 140.0;

/// Minimum time between state transitions (debouncing)
const MIN_STATE_CHANGE_MS: u64 = 100;

/// Panel state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelState {
    /// Panel not visible
    Hidden,
    /// Transitioning to visible
    Showing,
    /// Panel visible and ready
    Visible,
    /// Transitioning to hidden
    Hiding,
}

impl std::fmt::Display for PanelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelState::Hidden => write!(f, "Hidden"),
            PanelState::Showing => write!(f, "Showing"),
            PanelState::Visible => write!(f, "Visible"),
            PanelState::Hiding => write!(f, "Hiding"),
        }
    }
}

/// Result of a panel operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct PanelOperationResult {
    pub success: bool,
    pub state: String,
    pub error: Option<String>,
}

impl PanelOperationResult {
    pub fn success(state: PanelState) -> Self {
        Self {
            success: true,
            state: state.to_string(),
            error: None,
        }
    }

    pub fn error(state: PanelState, error: impl ToString) -> Self {
        Self {
            success: false,
            state: state.to_string(),
            error: Some(error.to_string()),
        }
    }
}

/// Quick capture panel state manager
pub struct QuickCaptureState {
    state: Mutex<PanelState>,
    last_change: Mutex<Instant>,
}

impl Default for QuickCaptureState {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickCaptureState {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(PanelState::Hidden),
            last_change: Mutex::new(Instant::now() - Duration::from_secs(1)),
        }
    }

    /// Get current state
    pub fn get_state(&self) -> PanelState {
        *self.state.lock().unwrap()
    }

    /// Try to transition to a new state, respecting the state machine rules
    pub fn try_transition(&self, target: PanelState) -> Result<PanelState, PanelState> {
        let mut state = self.state.lock().unwrap();
        let current = *state;

        // Check debouncing
        let mut last = self.last_change.lock().unwrap();
        if last.elapsed() < Duration::from_millis(MIN_STATE_CHANGE_MS) {
            return Err(current);
        }

        // Validate transition
        let valid = match (current, target) {
            (PanelState::Hidden, PanelState::Showing) => true,
            (PanelState::Showing, PanelState::Visible) => true,
            (PanelState::Visible, PanelState::Hiding) => true,
            (PanelState::Hiding, PanelState::Hidden) => true,
            (PanelState::Hidden, PanelState::Visible) => true,
            (PanelState::Showing, PanelState::Hiding) => true,
            (s, t) if s == t => true,
            _ => false,
        };

        if valid {
            *state = target;
            *last = Instant::now();
            Ok(current)
        } else {
            Err(current)
        }
    }

    /// Force state (for error recovery)
    pub fn force_state(&self, new_state: PanelState) {
        let mut state = self.state.lock().unwrap();
        let mut last = self.last_change.lock().unwrap();
        *state = new_state;
        *last = Instant::now();
    }
}

/// Show the quick capture window
/// Wrapped with catch_unwind since this is called from global hotkey handler
pub fn show(app: &AppHandle) -> Result<(), tauri::Error> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        show_impl(app)
    }));

    match result {
        Ok(inner_result) => inner_result,
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            tracing::error!("Panic in show_quick_capture: {}", msg);
            Err(tauri::Error::Anyhow(anyhow::anyhow!("Panic: {}", msg).into()))
        }
    }
}

fn show_impl(app: &AppHandle) -> Result<(), tauri::Error> {
    let state = get_or_create_state(app);

    // Try to transition to Showing state
    if let Err(current) = state.try_transition(PanelState::Showing) {
        if current == PanelState::Visible {
            return Ok(());
        }
        return Ok(());
    }

    // Get center position on current monitor
    let center_position = get_center_position(app);

    #[cfg(target_os = "macos")]
    {
        let result = show_macos(app, center_position);
        match &result {
            Ok(_) => {
                state.force_state(PanelState::Visible);
            }
            Err(e) => {
                tracing::error!("Failed to show quick capture: {}", e);
                state.force_state(PanelState::Hidden);
            }
        }
        return result;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let result = show_other(app, center_position);
        match &result {
            Ok(_) => {
                state.force_state(PanelState::Visible);
            }
            Err(e) => {
                tracing::error!("Failed to show quick capture: {}", e);
                state.force_state(PanelState::Hidden);
            }
        }
        result
    }
}

#[cfg(target_os = "macos")]
fn show_macos(
    app: &AppHandle,
    center_position: Option<PhysicalPosition<i32>>,
) -> Result<(), tauri::Error> {
    // Check if panel already exists in the panel manager
    if let Ok(panel) = app.get_webview_panel(QUICK_CAPTURE_LABEL) {
        // Reposition if needed
        if let Some(position) = center_position {
            if let Some(window) = panel.to_window() {
                let _ = window.set_position(position);
            }
        }
        // Show the panel without activating the app
        panel.order_front_regardless();
        panel.make_key_window();
        // Emit event to frontend
        let _ = app.emit("quick-capture:state-changed", "Visible");
        return Ok(());
    }

    // Check if window already exists but panel conversion failed previously
    // Instead of destroying, try to convert the existing window to a panel
    if let Some(existing_window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        // Reposition if needed
        if let Some(position) = center_position {
            let _ = existing_window.set_position(position);
        }

        // Try to convert existing window to panel
        if let Ok(panel) = existing_window.to_panel() {
            // Configure panel
            panel.set_level(PanelLevel::PopUpMenu.value());
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .can_join_all_spaces()
                    .stationary()
                    .ignores_cycle()
                    .full_screen_auxiliary()
                    .transient()
                    .into(),
            );
            panel.set_hides_on_deactivate(false);
            panel.order_front_regardless();
            panel.make_key_window();
            // Emit event to frontend
            let _ = app.emit("quick-capture:state-changed", "Visible");
            return Ok(());
        } else {
            // Destroy and wait
            let _ = existing_window.destroy();
            std::thread::sleep(Duration::from_millis(150));
        }
    }

    // Create a new window
    let mut builder = WebviewWindowBuilder::new(
        app,
        QUICK_CAPTURE_LABEL,
        WebviewUrl::App(QUICK_CAPTURE_URL.into()),
    )
    .title("Quick Capture")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .skip_taskbar(true)
    .always_on_top(true)
    .focused(false)
    .visible(false)
    .accept_first_mouse(true)
    .hidden_title(true)
    .title_bar_style(tauri::TitleBarStyle::Overlay);

    if let Some(position) = center_position {
        builder = builder.position(position.x as f64, position.y as f64);
    } else {
        builder = builder.center();
    }

    let window = builder.build()?;

    // Apply window effects for proper transparency - use HudWindow for floating panels
    let _ = window.set_effects(
        EffectsBuilder::new()
            .effects([Effect::HudWindow])
            .build(),
    );

    // Convert to panel with our custom panel type that has canBecomeKeyWindow: true
    let panel = window.to_panel().map_err(|e| {
        tracing::error!("Failed to convert window to panel: {:?}", e);
        tauri::Error::Anyhow(anyhow::anyhow!("Failed to convert to panel: {:?}", e).into())
    })?;

    // Configure panel level - PopUpMenu level (101) is ignored by tiling WMs
    panel.set_level(PanelLevel::PopUpMenu.value());

    // CRITICAL: Set non-activating style mask to prevent app activation
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

    // Configure collection behavior for proper window management
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .stationary()
            .ignores_cycle()
            .full_screen_auxiliary()
            .transient()
            .into(),
    );

    // Don't hide when app deactivates
    panel.set_hides_on_deactivate(false);

    // Show the panel
    panel.order_front_regardless();
    panel.make_key_window();

    // Emit event to frontend
    let _ = app.emit("quick-capture:state-changed", "Visible");

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn show_other(
    app: &AppHandle,
    center_position: Option<PhysicalPosition<i32>>,
) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        if let Some(position) = center_position {
            window.set_position(position)?;
        }
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(
        app,
        QUICK_CAPTURE_LABEL,
        WebviewUrl::App(QUICK_CAPTURE_URL.into()),
    )
    .title("Quick Capture")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .skip_taskbar(true)
    .always_on_top(true)
    .focused(true)
    .visible(true)
    .accept_first_mouse(true);

    if let Some(position) = center_position {
        builder = builder.position(position.x as f64, position.y as f64);
    } else {
        builder = builder.center();
    }

    builder.build()?;
    Ok(())
}

/// Hide the quick capture window
pub fn hide(app: &AppHandle) -> PanelOperationResult {
    let state = get_or_create_state(app);

    if let Err(current) = state.try_transition(PanelState::Hiding) {
        if current == PanelState::Hidden || current == PanelState::Hiding {
            return PanelOperationResult::success(current);
        }
        return PanelOperationResult::error(current, "Cannot hide from current state");
    }

    #[cfg(target_os = "macos")]
    {
        let result = hide_macos(app);
        state.force_state(PanelState::Hidden);
        match result {
            Ok(_) => PanelOperationResult::success(PanelState::Hidden),
            Err(e) => PanelOperationResult::error(PanelState::Hidden, e.to_string()),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let result = hide_other(app);
        state.force_state(PanelState::Hidden);
        match result {
            Ok(_) => PanelOperationResult::success(PanelState::Hidden),
            Err(e) => PanelOperationResult::error(PanelState::Hidden, e.to_string()),
        }
    }
}

#[cfg(target_os = "macos")]
fn hide_macos(app: &AppHandle) -> Result<(), tauri::Error> {
    // Use run_on_main_thread for thread safety
    let app_clone = app.clone();
    let app_for_emit = app.clone();
    app.run_on_main_thread(move || {
        // Try panel first, fall back to window
        let hidden = if let Ok(panel) = app_clone.get_webview_panel(QUICK_CAPTURE_LABEL) {
            panel.hide();
            true
        } else if let Some(window) = app_clone.get_webview_window(QUICK_CAPTURE_LABEL) {
            let _ = window.hide();
            true
        } else {
            false
        };

        if hidden {
            // Emit event only if we actually hid something
            let _ = app_for_emit.emit("quick-capture:state-changed", "Hidden");
        }
    })?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn hide_other(app: &AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        window.hide()?;
    }
    Ok(())
}

/// Get the center position for the quick capture window on the current monitor
fn get_center_position(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
    let monitors = app.available_monitors().ok()?;
    let primary_monitor = app.primary_monitor().ok().flatten();
    let cursor_pos = app.cursor_position().ok();

    let current_monitor = if let Some(cursor) = cursor_pos {
        monitors
            .iter()
            .find(|monitor| {
                let pos = monitor.position();
                let size = monitor.size();
                let right = pos.x + size.width as i32;
                let bottom = pos.y + size.height as i32;
                cursor.x >= pos.x as f64
                    && cursor.x < right as f64
                    && cursor.y >= pos.y as f64
                    && cursor.y < bottom as f64
            })
            .cloned()
            .or_else(|| primary_monitor.clone())
            .or_else(|| monitors.into_iter().next())
    } else {
        primary_monitor.or_else(|| monitors.into_iter().next())
    }?;

    let size = current_monitor.size();
    let position = current_monitor.position();

    let x = position.x + (size.width as i32 / 2) - (WINDOW_WIDTH as i32 / 2);
    let y = position.y + (size.height as i32 / 2) - (WINDOW_HEIGHT as i32 / 2);

    Some(PhysicalPosition::new(x, y))
}

fn get_or_create_state(app: &AppHandle) -> tauri::State<'_, QuickCaptureState> {
    app.state::<QuickCaptureState>()
}

pub fn get_state(app: &AppHandle) -> PanelState {
    let state = get_or_create_state(app);
    state.get_state()
}
