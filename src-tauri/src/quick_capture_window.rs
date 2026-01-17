use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

const QUICK_CAPTURE_LABEL: &str = "quick-capture";
const QUICK_CAPTURE_URL: &str = "index.html#/quick-capture";
const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 140.0;

pub fn show(app: &AppHandle) -> Result<(), tauri::Error> {
    // Get center position on current monitor FIRST
    let center_position = get_center_position(app);

    // If window already exists, reposition and show it
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        tracing::debug!("[QuickCapture] Window exists, repositioning and showing");

        // Reposition window on current monitor BEFORE showing
        if let Some(position) = center_position {
            tracing::debug!("[QuickCapture] Setting position to ({}, {})", position.x, position.y);
            window.set_position(position)?;
        }

        // Show and focus the window
        window.show()?;
        window.set_focus()?;

        tracing::info!("[QuickCapture] Window shown and focused");
        return Ok(());
    }

    // Build window (first time)
    tracing::debug!("[QuickCapture] Creating new window");

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
    .focused(true) // Request focus immediately
    .visible(false) // Start hidden to position first
    .accept_first_mouse(true) // Accept clicks without needing focus first
    .content_protected(true); // Prevent screen recording (also hints to WM to not manage)

    // On macOS, prevent window from auto-repositioning by tiling managers (Aerospace, yabai, etc.)
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .hidden_title(true)
            .title_bar_style(tauri::TitleBarStyle::Overlay); // Use overlay style to avoid WM interception
    }

    // Set position if we got one
    if let Some(position) = center_position {
        tracing::debug!("[QuickCapture] Setting initial position to ({}, {})", position.x, position.y);
        builder = builder.position(position.x as f64, position.y as f64);
    } else {
        tracing::warn!("[QuickCapture] No position calculated, using center");
        builder = builder.center();
    }

    let window = builder.build()?;

    tracing::debug!("[QuickCapture] Window built, configuring...");

    // On macOS, explicitly tell window managers to not manage this window
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
        use cocoa::base::id;

        unsafe {
            let ns_window = window.ns_window().unwrap() as id;

            // Set window level to floating (3 = NSFloatingWindowLevel)
            // This makes it float above normal windows and tells tiling managers to ignore it
            ns_window.setLevel_(3);

            // Tell window manager to not manage this window:
            // - CanJoinAllSpaces: appears on all spaces/desktops
            // - Stationary: doesn't move when switching spaces
            // - IgnoresCycle: excluded from Cmd+Tab and window cycling
            // - FullScreenAuxiliary: can appear alongside fullscreen windows
            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;

            ns_window.setCollectionBehavior_(behavior);

            tracing::debug!("[QuickCapture] Set macOS window level=3 and collection behavior to prevent tiling manager (Aerospace/yabai) interference");
        }
    }

    tracing::debug!("[QuickCapture] Showing window...");

    // Show and focus after positioning and configuration
    window.show()?;

    // Verify position after showing
    if let Ok(actual_position) = window.outer_position() {
        tracing::info!(
            "[QuickCapture] Window shown. Actual position: ({}, {})",
            actual_position.x,
            actual_position.y
        );
    }

    window.set_focus()?;

    // Clone window for async check
    let window_clone = window.clone();
    let target_pos = center_position;

    // Check if position changes after a delay (indicates WM interference)
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Ok(actual_position) = window_clone.outer_position() {
            if let Some(target) = target_pos {
                let x_diff = (actual_position.x - target.x).abs();
                let y_diff = (actual_position.y - target.y).abs();
                if x_diff > 10 || y_diff > 10 {
                    tracing::warn!(
                        "[QuickCapture] Window was MOVED by external force! Target: ({}, {}), Actual: ({}, {})",
                        target.x,
                        target.y,
                        actual_position.x,
                        actual_position.y
                    );
                } else {
                    tracing::info!("[QuickCapture] Position stable at ({}, {})", actual_position.x, actual_position.y);
                }
            }
        }
    });

    tracing::info!("[QuickCapture] Window created, shown, and focused");

    Ok(())
}

/// Get the center position for the quick capture window on the current monitor
fn get_center_position(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
    let monitors = app.available_monitors().ok()?;
    let primary_monitor = app.primary_monitor().ok().flatten();

    // Try to get cursor position
    let cursor_pos = app.cursor_position().ok();
    tracing::debug!("[QuickCapture] Cursor position: {:?}", cursor_pos);

    for monitor in &monitors {
        let pos = monitor.position();
        let size = monitor.size();
        let name = monitor.name().map(|name| name.as_str()).unwrap_or("unknown");
        tracing::debug!(
            "[QuickCapture] Monitor '{}' pos=({}, {}), size={}x{}, scale_factor={}",
            name,
            pos.x,
            pos.y,
            size.width,
            size.height,
            monitor.scale_factor()
        );
    }

    let (current_monitor, selection_method) = if let Some(cursor) = cursor_pos {
        // Use physical bounds to avoid logical/scale mismatches.
        if let Some(monitor) = monitors.iter().find(|monitor| {
            let pos = monitor.position();
            let size = monitor.size();
            let right = pos.x + size.width as i32;
            let bottom = pos.y + size.height as i32;
            let left = pos.x as f64;
            let top = pos.y as f64;
            let right = right as f64;
            let bottom = bottom as f64;
            cursor.x >= left && cursor.x < right && cursor.y >= top && cursor.y < bottom
        }) {
            tracing::info!(
                "[QuickCapture] Selected monitor under cursor at ({}, {})",
                cursor.x,
                cursor.y
            );
            (monitor.clone(), "cursor-physical")
        } else if let Some(monitor) = primary_monitor.clone() {
            tracing::info!("[QuickCapture] Cursor monitor not found, using primary monitor");
            (monitor, "primary")
        } else {
            let monitor = monitors.into_iter().next()?;
            tracing::info!("[QuickCapture] Using first available monitor as fallback");
            (monitor, "fallback")
        }
    } else if let Some(monitor) = primary_monitor.clone() {
        tracing::info!("[QuickCapture] Cursor position unavailable, using primary monitor");
        (monitor, "primary")
    } else {
        let monitor = monitors.into_iter().next()?;
        tracing::info!("[QuickCapture] Using first available monitor as fallback");
        (monitor, "fallback")
    };

    let size = current_monitor.size();
    let position = current_monitor.position();

    tracing::info!(
        "[QuickCapture] Monitor selected via '{}': position=({}, {}), size={}x{}",
        selection_method,
        position.x,
        position.y,
        size.width,
        size.height
    );

    // Calculate center position
    let x = position.x + (size.width as i32 / 2) - (WINDOW_WIDTH as i32 / 2);
    let y = position.y + (size.height as i32 / 2) - (WINDOW_HEIGHT as i32 / 2);

    tracing::info!(
        "[QuickCapture] Window will be positioned at ({}, {})",
        x,
        y
    );

    Some(PhysicalPosition::new(x, y))
}

pub fn hide(app: &AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        let _ = window.hide();
    }

    Ok(())
}
