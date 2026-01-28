//! macOS NSPanel implementation
//!
//! Provides a minimal NSPanel wrapper for floating windows that:
//! - Can become key window (receive keyboard input)
//! - Float above other windows
//! - Don't activate the main application

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObjectProtocol};
use objc2::{define_class, msg_send, ClassType};
use objc2_app_kit::{NSPanel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};
use objc2_foundation::NSObject;
use tauri::{plugin::TauriPlugin, Manager, Runtime, WebviewWindow};

// ============================================================================
// Panel Level
// ============================================================================

/// Window level constants for NSPanel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelLevel {
    /// Normal window level (0)
    Normal,
    /// Floating window level (4)
    Floating,
    /// Pop-up menu window level (101) - ideal for floating panels
    PopUpMenu,
    /// Custom level value
    Custom(i32),
}

impl PanelLevel {
    /// Convert to the raw i64 value used by NSWindow
    pub fn value(&self) -> i64 {
        match self {
            PanelLevel::Normal => 0,
            PanelLevel::Floating => 4,
            PanelLevel::PopUpMenu => 101,
            PanelLevel::Custom(value) => *value as i64,
        }
    }
}

// ============================================================================
// Collection Behavior Builder
// ============================================================================

/// Builder for NSWindowCollectionBehavior flags
#[derive(Debug, Clone, Copy)]
pub struct CollectionBehavior(NSWindowCollectionBehavior);

impl Default for CollectionBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionBehavior {
    pub fn new() -> Self {
        Self(NSWindowCollectionBehavior::empty())
    }

    /// Window can be shown on all spaces
    pub fn can_join_all_spaces(mut self) -> Self {
        self.0 |= NSWindowCollectionBehavior::CanJoinAllSpaces;
        self
    }

    /// Window does not participate in Spaces or Expose
    pub fn stationary(mut self) -> Self {
        self.0 |= NSWindowCollectionBehavior::Stationary;
        self
    }

    /// Window ignores Cmd+Tab cycling
    pub fn ignores_cycle(mut self) -> Self {
        self.0 |= NSWindowCollectionBehavior::IgnoresCycle;
        self
    }

    /// Window can be shown alongside full screen window
    pub fn full_screen_auxiliary(mut self) -> Self {
        self.0 |= NSWindowCollectionBehavior::FullScreenAuxiliary;
        self
    }

    /// Window participates in Spaces and Expose (transient)
    pub fn transient(mut self) -> Self {
        self.0 |= NSWindowCollectionBehavior::Transient;
        self
    }
}

impl From<CollectionBehavior> for NSWindowCollectionBehavior {
    fn from(cb: CollectionBehavior) -> Self {
        cb.0
    }
}

// ============================================================================
// Style Mask Builder
// ============================================================================

/// Builder for NSWindowStyleMask flags
#[derive(Debug, Clone, Copy)]
pub struct StyleMask(NSWindowStyleMask);

impl Default for StyleMask {
    fn default() -> Self {
        Self::empty()
    }
}

impl StyleMask {
    pub fn empty() -> Self {
        Self(NSWindowStyleMask::empty())
    }

    /// Non-activating panel - doesn't activate the application when shown
    pub fn nonactivating_panel(mut self) -> Self {
        self.0 |= NSWindowStyleMask::NonactivatingPanel;
        self
    }
}

impl From<StyleMask> for NSWindowStyleMask {
    fn from(sm: StyleMask) -> Self {
        sm.0
    }
}

// ============================================================================
// Custom Panel Class Definition
// ============================================================================

/// Instance variables for our custom panel class
struct QuickCapturePanelIvars;

define_class!(
    #[unsafe(super = NSPanel)]
    #[name = "QuickCapturePanel"]
    #[ivars = QuickCapturePanelIvars]
    struct RawQuickCapturePanel;

    unsafe impl NSObjectProtocol for RawQuickCapturePanel {}

    impl RawQuickCapturePanel {
        /// Override canBecomeKeyWindow to return true
        /// This allows the panel to receive keyboard input
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            true
        }

        /// Override canBecomeMainWindow to return false
        /// Panel should not become the main window
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }
    }
);

// ============================================================================
// Panel Wrapper
// ============================================================================

/// A wrapper around NSPanel that provides a safe interface
pub struct QuickCapturePanel<R: Runtime = tauri::Wry> {
    panel: Retained<RawQuickCapturePanel>,
    label: String,
    original_class: *const AnyClass,
    app_handle: tauri::AppHandle<R>,
}

// SAFETY: The panel must only be used on the main thread, but we implement
// Send + Sync to allow passing references through Tauri's state system.
// All actual panel operations use run_on_main_thread.
unsafe impl<R: Runtime> Send for QuickCapturePanel<R> {}
unsafe impl<R: Runtime> Sync for QuickCapturePanel<R> {}

impl<R: Runtime> QuickCapturePanel<R> {
    /// Show the panel (orderFrontRegardless)
    pub fn show(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderFrontRegardless];
        }
    }

    /// Hide the panel (orderOut:)
    pub fn hide(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderOut: std::ptr::null::<AnyObject>()];
        }
    }

    /// Make the panel key window (receives keyboard input)
    pub fn make_key_window(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, makeKeyWindow];
        }
    }

    /// Order front regardless of app activation state
    pub fn order_front_regardless(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderFrontRegardless];
        }
    }

    /// Set the window level
    pub fn set_level(&self, level: i64) {
        unsafe {
            let _: () = msg_send![&*self.panel, setLevel: level];
        }
    }

    /// Set the style mask
    pub fn set_style_mask(&self, style_mask: NSWindowStyleMask) {
        unsafe {
            let _: () = msg_send![&*self.panel, setStyleMask: style_mask];
        }
    }

    /// Set collection behavior
    pub fn set_collection_behavior(&self, behavior: NSWindowCollectionBehavior) {
        unsafe {
            let _: () = msg_send![&*self.panel, setCollectionBehavior: behavior];
        }
    }

    /// Set whether the panel hides when the app deactivates
    pub fn set_hides_on_deactivate(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setHidesOnDeactivate: value];
        }
    }

    /// Get the label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Convert back to a regular Tauri window (removes from panel manager)
    pub fn to_window(&self) -> Option<WebviewWindow<R>> {
        unsafe extern "C" {
            fn object_setClass(
                obj: *mut NSObject,
                cls: *const AnyClass,
            ) -> *const AnyClass;
        }

        // Remove from panel manager
        if self.app_handle.remove_webview_panel(&self.label).is_some() {
            // Restore original class
            unsafe {
                let target_class = if !self.original_class.is_null() {
                    self.original_class
                } else {
                    NSWindow::class()
                };

                object_setClass(
                    &*self.panel as *const RawQuickCapturePanel as *mut NSObject,
                    target_class,
                );
            }

            self.app_handle.get_webview_window(&self.label)
        } else {
            None
        }
    }
}

// ============================================================================
// Panel Manager
// ============================================================================

/// Type alias for shared panel references
pub type PanelHandle<R> = Arc<dyn Panel<R>>;

/// Trait for panel operations (allows type erasure)
pub trait Panel<R: Runtime>: Send + Sync {
    fn show(&self);
    fn hide(&self);
    fn make_key_window(&self);
    fn order_front_regardless(&self);
    fn set_level(&self, level: i64);
    fn set_style_mask(&self, style_mask: NSWindowStyleMask);
    fn set_collection_behavior(&self, behavior: NSWindowCollectionBehavior);
    fn set_hides_on_deactivate(&self, value: bool);
    fn label(&self) -> &str;
    fn to_window(&self) -> Option<WebviewWindow<R>>;
}

impl<R: Runtime> Panel<R> for QuickCapturePanel<R> {
    fn show(&self) {
        QuickCapturePanel::show(self)
    }

    fn hide(&self) {
        QuickCapturePanel::hide(self)
    }

    fn make_key_window(&self) {
        QuickCapturePanel::make_key_window(self)
    }

    fn order_front_regardless(&self) {
        QuickCapturePanel::order_front_regardless(self)
    }

    fn set_level(&self, level: i64) {
        QuickCapturePanel::set_level(self, level)
    }

    fn set_style_mask(&self, style_mask: NSWindowStyleMask) {
        QuickCapturePanel::set_style_mask(self, style_mask)
    }

    fn set_collection_behavior(&self, behavior: NSWindowCollectionBehavior) {
        QuickCapturePanel::set_collection_behavior(self, behavior)
    }

    fn set_hides_on_deactivate(&self, value: bool) {
        QuickCapturePanel::set_hides_on_deactivate(self, value)
    }

    fn label(&self) -> &str {
        QuickCapturePanel::label(self)
    }

    fn to_window(&self) -> Option<WebviewWindow<R>> {
        QuickCapturePanel::to_window(self)
    }
}

/// Storage for panels
struct PanelStore<R: Runtime> {
    panels: HashMap<String, PanelHandle<R>>,
}

impl<R: Runtime> Default for PanelStore<R> {
    fn default() -> Self {
        Self {
            panels: HashMap::new(),
        }
    }
}

/// Panel manager state (stored in Tauri app state)
pub struct PanelManager<R: Runtime>(Mutex<PanelStore<R>>);

impl<R: Runtime> Default for PanelManager<R> {
    fn default() -> Self {
        Self(Mutex::new(PanelStore::default()))
    }
}

// ============================================================================
// Extension Traits
// ============================================================================

#[derive(Debug)]
pub enum PanelError {
    PanelNotFound,
}

impl std::fmt::Display for PanelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelError::PanelNotFound => write!(f, "Panel not found"),
        }
    }
}

impl std::error::Error for PanelError {}

/// Extension trait for Tauri Manager to access panels
pub trait ManagerExt<R: Runtime> {
    fn get_webview_panel(&self, label: &str) -> Result<PanelHandle<R>, PanelError>;
    fn remove_webview_panel(&self, label: &str) -> Option<PanelHandle<R>>;
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
    fn get_webview_panel(&self, label: &str) -> Result<PanelHandle<R>, PanelError> {
        let manager = self.state::<PanelManager<R>>();
        let store = manager.0.lock().unwrap();

        store
            .panels
            .get(label)
            .cloned()
            .ok_or(PanelError::PanelNotFound)
    }

    fn remove_webview_panel(&self, label: &str) -> Option<PanelHandle<R>> {
        let manager = self.state::<PanelManager<R>>();
        let mut store = manager.0.lock().unwrap();
        store.panels.remove(label)
    }
}

/// Extension trait for WebviewWindow to convert to panel
pub trait WebviewWindowExt<R: Runtime> {
    fn to_panel(&self) -> tauri::Result<PanelHandle<R>>;
}

impl<R: Runtime> WebviewWindowExt<R> for WebviewWindow<R> {
    fn to_panel(&self) -> tauri::Result<PanelHandle<R>> {
        unsafe extern "C" {
            fn object_setClass(
                obj: *mut NSObject,
                cls: *const AnyClass,
            ) -> *const AnyClass;

            fn object_getClass(obj: *mut NSObject) -> *const AnyClass;
        }

        let label = self.label().to_string();

        // Get the NSWindow pointer from Tauri
        let ns_window = self.ns_window().map_err(|e| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get NSWindow: {:?}", e),
            ))
        })?;

        unsafe {
            // Save original class for restoration
            let original_class = object_getClass(ns_window as *mut NSObject);

            // Swizzle the class to our custom panel class
            object_setClass(ns_window as *mut NSObject, RawQuickCapturePanel::class());

            // Cast to our panel type
            let panel_ptr = ns_window as *mut RawQuickCapturePanel;

            // Create a Retained reference
            let panel = Retained::retain(panel_ptr).ok_or_else(|| {
                tauri::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to retain panel",
                ))
            })?;

            // Set as floating panel
            let _: () = msg_send![&*panel, setFloatingPanel: true];

            // Create wrapper
            let quick_capture_panel = QuickCapturePanel {
                panel,
                label: label.clone(),
                original_class,
                app_handle: self.app_handle().clone(),
            };

            let arc_panel = Arc::new(quick_capture_panel) as PanelHandle<R>;

            // Store in panel manager
            let manager = self.state::<PanelManager<R>>();
            manager.0.lock().unwrap().panels.insert(label, arc_panel.clone());

            Ok(arc_panel)
        }
    }
}

// ============================================================================
// Plugin Initialization
// ============================================================================

/// Initialize the NSPanel plugin (registers PanelManager in app state)
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("nspanel")
        .setup(|app, _api| {
            app.manage(PanelManager::<R>::default());
            Ok(())
        })
        .build()
}
