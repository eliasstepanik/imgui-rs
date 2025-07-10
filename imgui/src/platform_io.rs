//! Platform I/O and viewport support for ImGui.
//!
//! # Viewport Support
//!
//! Dear ImGui supports multiple viewports, allowing ImGui windows to be dragged outside
//! the main application window and rendered as separate OS windows. This feature requires
//! the `docking` feature to be enabled.
//!
//! ## Viewport Lifecycle
//!
//! 1. **Creation**: When an ImGui window is dragged outside the main window, ImGui creates
//!    a new viewport and calls `PlatformViewportBackend::create_window()`.
//!
//! 2. **Rendering**: Each frame, viewports with visible content will have their `DrawData`
//!    populated. Use `viewport.draw_data()` to safely access it (returns `None` if no content).
//!
//! 3. **Updates**: Platform backends handle window movement, resizing, and other OS events
//!    through the various `PlatformViewportBackend` methods.
//!
//! 4. **Destruction**: When a viewport is no longer needed, ImGui calls
//!    `PlatformViewportBackend::destroy_window()`.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "docking")] {
//! # use imgui::*;
//! # fn render_draw_data(draw_data: &DrawData) {}
//! # let mut ctx = Context::create();
//! # // Initialize context properly
//! # ctx.io_mut().display_size = [1280.0, 720.0];
//! # ctx.io_mut().delta_time = 1.0 / 60.0;
//! # ctx.fonts().build_rgba32_texture();
//! // Enable viewport support
//! ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
//!
//! // In your render loop:
//! ctx.new_frame();
//! // ... build UI here ...
//! let draw_data = ctx.render(); // Must call render() first
//! render_draw_data(draw_data);
//!
//! ctx.update_platform_windows(); // Call AFTER render()
//!
//! // Render additional viewports
//! for viewport in ctx.viewports() {
//!     if !viewport.is_main() {
//!         if let Some(draw_data) = viewport.draw_data() {
//!             render_draw_data(draw_data);
//!         }
//!     }
//! }
//!
//! ctx.render_platform_windows_default();
//! # }
//! ```
//!
//! ## Safety Notes
//!
//! - `DrawData` is only valid between `render()` and the next `new_frame()`
//! - Viewports can be created/destroyed at any time by ImGui
//! - Platform handles must be validated before use

use std::ffi::{c_char, c_void};

use crate::{internal::RawCast, ViewportFlags};

#[cfg(feature = "docking")]
use crate::{internal::ImVector, PlatformMonitor};

/// Holds the information needed to enable multiple viewports.
#[repr(C)]
pub struct PlatformIo {
    pub(crate) get_clipboard_text_fn:
        Option<unsafe extern "C" fn(*mut sys::ImGuiContext) -> *const c_char>,

    pub(crate) set_clipboard_text_fn:
        Option<unsafe extern "C" fn(*mut sys::ImGuiContext, *const c_char)>,

    pub(crate) clipboard_user_data: *mut c_void,

    pub(crate) open_in_shell_fn:
        Option<unsafe extern "C" fn(ctx: *mut sys::ImGuiContext, path: *const c_char) -> bool>,
    pub(crate) open_in_shell_user_data: *mut c_void,
    pub(crate) set_ime_data_fn: Option<
        unsafe extern "C" fn(
            ctx: *mut sys::ImGuiContext,
            viewport: *mut sys::ImGuiViewport,
            data: *mut sys::ImGuiPlatformImeData,
        ),
    >,
    pub(crate) ime_user_data: *mut c_void,
    pub(crate) locale_decimal_point: sys::ImWchar,

    #[cfg(feature = "docking")]
    pub(crate) platform_create_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_destroy_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_show_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_set_window_pos: Option<unsafe extern "C" fn(*mut Viewport, sys::ImVec2)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_pos: Option<unsafe extern "C" fn(*mut Viewport) -> sys::ImVec2>,
    #[cfg(feature = "docking")]
    pub(crate) platform_set_window_size: Option<unsafe extern "C" fn(*mut Viewport, sys::ImVec2)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_size: Option<unsafe extern "C" fn(*mut Viewport) -> sys::ImVec2>,
    #[cfg(feature = "docking")]
    pub(crate) platform_set_window_focus: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_focus: Option<unsafe extern "C" fn(*mut Viewport) -> bool>,
    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_minimized: Option<unsafe extern "C" fn(*mut Viewport) -> bool>,
    #[cfg(feature = "docking")]
    pub(crate) platform_set_window_title:
        Option<unsafe extern "C" fn(*mut Viewport, *const c_char)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_set_window_alpha: Option<unsafe extern "C" fn(*mut Viewport, f32)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_update_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_render_window: Option<unsafe extern "C" fn(*mut Viewport, *mut c_void)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_swap_buffers: Option<unsafe extern "C" fn(*mut Viewport, *mut c_void)>,
    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_dpi_scale: Option<unsafe extern "C" fn(*mut Viewport) -> f32>,
    #[cfg(feature = "docking")]
    pub(crate) platform_on_changed_viewport: Option<unsafe extern "C" fn(*mut Viewport)>,

    #[cfg(feature = "docking")]
    pub(crate) platform_get_window_work_area_inserts:
        Option<unsafe extern "C" fn(vp: *mut Viewport) -> sys::ImVec4>,

    #[cfg(feature = "docking")]
    pub(crate) platform_create_vk_surface: Option<
        unsafe extern "C" fn(*mut Viewport, u64, *const c_void, *mut u64) -> std::ffi::c_int,
    >,

    #[cfg(feature = "docking")]
    pub(crate) renderer_create_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) renderer_destroy_window: Option<unsafe extern "C" fn(*mut Viewport)>,
    #[cfg(feature = "docking")]
    pub(crate) renderer_set_window_size: Option<unsafe extern "C" fn(*mut Viewport, sys::ImVec2)>,
    #[cfg(feature = "docking")]
    pub(crate) renderer_render_window: Option<unsafe extern "C" fn(*mut Viewport, *mut c_void)>,
    #[cfg(feature = "docking")]
    pub(crate) renderer_swap_buffers: Option<unsafe extern "C" fn(*mut Viewport, *mut c_void)>,

    /// Holds information about the available monitors.
    /// Should be initialized and updated by the [`PlatformViewportBackend`](crate::PlatformViewportBackend).
    #[cfg(feature = "docking")]
    pub monitors: ImVector<PlatformMonitor>,

    #[cfg(feature = "docking")]
    pub(crate) viewports: ImVector<*mut Viewport>,
}

unsafe impl RawCast<sys::ImGuiPlatformIO> for PlatformIo {}

#[test]
#[cfg(test)]
fn test_platform_io_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<PlatformIo>(),
        mem::size_of::<sys::ImGuiPlatformIO>()
    );
    assert_eq!(
        mem::align_of::<PlatformIo>(),
        mem::align_of::<sys::ImGuiPlatformIO>()
    );
    use sys::ImGuiPlatformIO;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(PlatformIo, $l),
                memoffset::offset_of!(ImGuiPlatformIO, $r)
            );
        };
    }

    assert_field_offset!(get_clipboard_text_fn, Platform_GetClipboardTextFn);
    assert_field_offset!(set_clipboard_text_fn, Platform_SetClipboardTextFn);
    assert_field_offset!(clipboard_user_data, Platform_ClipboardUserData);
    assert_field_offset!(open_in_shell_fn, Platform_OpenInShellFn);
    assert_field_offset!(open_in_shell_user_data, Platform_OpenInShellUserData);
    assert_field_offset!(set_ime_data_fn, Platform_SetImeDataFn);
    assert_field_offset!(ime_user_data, Platform_ImeUserData);
    assert_field_offset!(locale_decimal_point, Platform_LocaleDecimalPoint);

    #[cfg(feature = "docking")]
    {
        assert_field_offset!(platform_create_window, Platform_CreateWindow);
        assert_field_offset!(platform_destroy_window, Platform_DestroyWindow);
        assert_field_offset!(platform_show_window, Platform_ShowWindow);
        assert_field_offset!(platform_set_window_pos, Platform_SetWindowPos);
        assert_field_offset!(platform_get_window_pos, Platform_GetWindowPos);
        assert_field_offset!(platform_set_window_size, Platform_SetWindowSize);
        assert_field_offset!(platform_get_window_size, Platform_GetWindowSize);
        assert_field_offset!(platform_set_window_focus, Platform_SetWindowFocus);
        assert_field_offset!(platform_get_window_focus, Platform_GetWindowFocus);
        assert_field_offset!(platform_get_window_minimized, Platform_GetWindowMinimized);
        assert_field_offset!(platform_set_window_title, Platform_SetWindowTitle);
        assert_field_offset!(platform_set_window_alpha, Platform_SetWindowAlpha);
        assert_field_offset!(platform_update_window, Platform_UpdateWindow);
        assert_field_offset!(platform_render_window, Platform_RenderWindow);
        assert_field_offset!(platform_swap_buffers, Platform_SwapBuffers);
        assert_field_offset!(platform_get_window_dpi_scale, Platform_GetWindowDpiScale);
        assert_field_offset!(platform_on_changed_viewport, Platform_OnChangedViewport);
        assert_field_offset!(platform_create_vk_surface, Platform_CreateVkSurface);

        assert_field_offset!(renderer_create_window, Renderer_CreateWindow);
        assert_field_offset!(renderer_destroy_window, Renderer_DestroyWindow);
        assert_field_offset!(renderer_set_window_size, Renderer_SetWindowSize);
        assert_field_offset!(renderer_render_window, Renderer_RenderWindow);
        assert_field_offset!(renderer_swap_buffers, Renderer_SwapBuffers);

        assert_field_offset!(monitors, Monitors);
        assert_field_offset!(viewports, Viewports);
    }
}

/// Describes an ImGui Viewport.
#[repr(C)]
pub struct Viewport {
    /// The unique ID of this Viewport.
    pub id: crate::Id,
    /// Flags that describe how the Viewport should behave.
    pub flags: ViewportFlags,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub work_pos: [f32; 2],
    pub work_size: [f32; 2],
    #[cfg(feature = "docking")]
    pub dpi_scale: f32,

    #[cfg(feature = "docking")]
    pub(crate) parent_viewport_id: crate::Id,
    #[cfg(feature = "docking")]
    pub(crate) draw_data: *mut crate::DrawData,

    #[cfg(feature = "docking")]
    pub renderer_user_data: *mut c_void,
    #[cfg(feature = "docking")]
    pub platform_user_data: *mut c_void,

    pub platform_handle: *mut c_void,
    pub platform_handle_raw: *mut c_void,

    #[cfg(feature = "docking")]
    pub platform_window_created: bool,
    #[cfg(feature = "docking")]
    pub platform_request_move: bool,
    #[cfg(feature = "docking")]
    pub platform_request_resize: bool,
    #[cfg(feature = "docking")]
    pub platform_request_close: bool,
}

#[cfg(feature = "docking")]
impl Viewport {
    /// Returns the draw data of the viewport if it has any content to render.
    /// 
    /// Returns `None` if the viewport has no visible content or DrawData is not available.
    pub fn draw_data(&self) -> Option<&crate::DrawData> {
        if self.draw_data.is_null() {
            None
        } else {
            Some(unsafe { &*self.draw_data })
        }
    }
    
    /// Returns true if this is the main viewport (the primary application window).
    /// 
    /// The main viewport always exists and has a special ID of 0x11111111.
    pub fn is_main(&self) -> bool {
        // Main viewport has the special ID defined in imgui
        const IMGUI_VIEWPORT_DEFAULT_ID: u32 = 0x11111111;
        self.id.0 == IMGUI_VIEWPORT_DEFAULT_ID
    }
    
    /// Returns the platform handle cast to the specified type.
    /// 
    /// # Safety
    /// The caller must ensure the platform handle is valid and of the correct type.
    pub unsafe fn platform_handle_as<T>(&self) -> Option<&T> {
        if self.platform_handle.is_null() {
            None
        } else {
            Some(&*(self.platform_handle as *const T))
        }
    }
}

#[test]
#[cfg(test)]
fn test_viewport_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<Viewport>(),
        mem::size_of::<sys::ImGuiViewport>()
    );
    assert_eq!(
        mem::align_of::<Viewport>(),
        mem::align_of::<sys::ImGuiViewport>()
    );
    use sys::ImGuiViewport;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(Viewport, $l),
                memoffset::offset_of!(ImGuiViewport, $r)
            );
        };
    }

    assert_field_offset!(id, ID);
    assert_field_offset!(flags, Flags);
    assert_field_offset!(pos, Pos);
    assert_field_offset!(size, Size);
    assert_field_offset!(work_pos, WorkPos);
    assert_field_offset!(work_size, WorkSize);
    assert_field_offset!(platform_handle, PlatformHandle);
    assert_field_offset!(platform_handle_raw, PlatformHandleRaw);

    #[cfg(feature = "docking")]
    {
        assert_field_offset!(dpi_scale, DpiScale);
        assert_field_offset!(parent_viewport_id, ParentViewportId);
        assert_field_offset!(draw_data, DrawData);

        assert_field_offset!(renderer_user_data, RendererUserData);
        assert_field_offset!(platform_user_data, PlatformUserData);
        assert_field_offset!(platform_window_created, PlatformWindowCreated);
        assert_field_offset!(platform_request_move, PlatformRequestMove);
        assert_field_offset!(platform_request_resize, PlatformRequestResize);
        assert_field_offset!(platform_request_close, PlatformRequestClose);
    }
}

#[test]
#[cfg(all(test, feature = "docking"))]
fn test_viewport_is_main() {
    // Use the test helper that properly initializes the context
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    
    // Must be within a frame to access viewport data
    ctx.new_frame();
    
    // Test that main viewport is correctly identified
    let main_viewport = ctx.main_viewport();
    assert!(main_viewport.is_main());
    assert_eq!(main_viewport.id.0, 0x11111111);
}

#[test]
#[cfg(all(test, feature = "docking"))]
fn test_viewport_draw_data_null_safety() {
    // Use the test helper that properly initializes the context
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    
    // Must be within a frame to access viewport data
    ctx.new_frame();
    
    // Without rendering, viewports might have null draw data
    // This test ensures draw_data() doesn't panic on null pointers
    for viewport in ctx.viewports() {
        let _draw_data = viewport.draw_data(); // Should not panic
    }
}

#[test]
#[cfg(all(test, feature = "docking"))]
fn test_viewport_platform_handle_null_safety() {
    use std::ffi::c_void;
    
    // Create a mock viewport with null platform handle
    let viewport = Viewport {
        id: crate::Id(123),
        flags: ViewportFlags::empty(),
        pos: [0.0, 0.0],
        size: [100.0, 100.0],
        work_pos: [0.0, 0.0],
        work_size: [100.0, 100.0],
        dpi_scale: 1.0,
        parent_viewport_id: crate::Id(0),
        draw_data: std::ptr::null_mut(),
        renderer_user_data: std::ptr::null_mut(),
        platform_user_data: std::ptr::null_mut(),
        platform_handle: std::ptr::null_mut(),
        platform_handle_raw: std::ptr::null_mut(),
        platform_window_created: false,
        platform_request_move: false,
        platform_request_resize: false,
        platform_request_close: false,
    };
    
    // Test that platform_handle_as returns None for null handle
    unsafe {
        let handle: Option<&c_void> = viewport.platform_handle_as::<c_void>();
        assert!(handle.is_none());
    }
}

#[test]
#[cfg(all(test, feature = "docking"))]
fn test_viewport_iteration() {
    // Use the test helper that properly initializes the context
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    
    // Must be within a frame to access viewport data
    ctx.new_frame();
    
    // At minimum, there should be one viewport (the main viewport)
    let viewport_count = ctx.viewports().count();
    assert!(viewport_count >= 1);
    
    // The main viewport should be in the iteration
    let has_main = ctx.viewports().any(|vp| vp.is_main());
    assert!(has_main);
}
