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
        // Reposition window on current monitor BEFORE showing
        if let Some(position) = center_position {
            window.set_position(position)?;
        }

        // Show and focus the window
        window.show()?;
        window.set_focus()?;

        return Ok(());
    }

    // Build window (first time)

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
        builder = builder.position(position.x as f64, position.y as f64);
    } else {
        builder = builder.center();
    }

    let window = builder.build()?;

    // On macOS, explicitly tell window managers to not manage this window
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
        use cocoa::base::id;

        unsafe {
            let ns_window = window.ns_window().unwrap() as id;

            // Set window level to floating (3 = NSFloatingWindowLevel)
            ns_window.setLevel_(3);

            // CRITICAL: Prevent this window from activating the app
            // This keeps other apps in focus and prevents main window from coming forward
            ns_window.setHidesOnDeactivate_(cocoa::base::NO);

            // Tell window manager to not manage this window
            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;

            ns_window.setCollectionBehavior_(behavior);
        }
    }

    // Show the window WITHOUT activating the app
    // This is critical to prevent the main window from coming to focus
    window.show()?;

    // Focus the window, but don't activate the entire app
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSWindow;
        use cocoa::base::id;

        unsafe {
            let ns_window = window.ns_window().unwrap() as id;
            // Make key window without activating app (keeps other apps in focus)
            ns_window.makeKeyWindow();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        window.set_focus()?;
    }

    Ok(())
}

/// Get the center position for the quick capture window on the current monitor
fn get_center_position(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
    let monitors = app.available_monitors().ok()?;
    let primary_monitor = app.primary_monitor().ok().flatten();

    // Try to get cursor position
    let cursor_pos = app.cursor_position().ok();

    let current_monitor = if let Some(cursor) = cursor_pos {
        // Use physical bounds to avoid logical/scale mismatches
        monitors.iter().find(|monitor| {
            let pos = monitor.position();
            let size = monitor.size();
            let right = pos.x + size.width as i32;
            let bottom = pos.y + size.height as i32;
            let left = pos.x as f64;
            let top = pos.y as f64;
            let right = right as f64;
            let bottom = bottom as f64;
            cursor.x >= left && cursor.x < right && cursor.y >= top && cursor.y < bottom
        })
        .cloned()
        .or_else(|| primary_monitor.clone())
        .or_else(|| monitors.into_iter().next())
    } else {
        primary_monitor.clone().or_else(|| monitors.into_iter().next())
    }?;

    let size = current_monitor.size();
    let position = current_monitor.position();

    // Calculate center position
    let x = position.x + (size.width as i32 / 2) - (WINDOW_WIDTH as i32 / 2);
    let y = position.y + (size.height as i32 / 2) - (WINDOW_HEIGHT as i32 / 2);

    Some(PhysicalPosition::new(x, y))
}

pub fn hide(app: &AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_LABEL) {
        let _ = window.hide();
    }

    Ok(())
}
