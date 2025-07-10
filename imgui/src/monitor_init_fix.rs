//! Monitor initialization fix for viewport support.
//!
//! This module provides a workaround for the missing monitor initialization
//! in platform backends like imgui-winit-support. Without proper monitor
//! initialization, Dear ImGui throws an assertion error: "Platform init
//! didn't setup Monitors list?" when viewports are enabled.
//!
//! # Usage
//!
//! ```rust,no_run
//! use imgui::*;
//!
//! let mut ctx = Context::create();
//! ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
//!
//! // Initialize monitors with data from your windowing system
//! monitor_init_fix::init_monitors(&mut ctx, &[
//!     MonitorData {
//!         position: [0.0, 0.0],
//!         size: [1920.0, 1080.0],
//!         work_pos: [0.0, 0.0],
//!         work_size: [1920.0, 1040.0], // Excluding taskbar
//!         dpi_scale: 1.0,
//!     },
//!     MonitorData {
//!         position: [1920.0, 0.0],
//!         size: [1920.0, 1080.0],
//!         work_pos: [1920.0, 0.0],
//!         work_size: [1920.0, 1080.0],
//!         dpi_scale: 1.0,
//!     },
//! ]);
//! ```

use crate::{Context, PlatformMonitor};

/// Monitor data for initialization.
#[derive(Debug, Clone)]
pub struct MonitorData {
    /// Position of the monitor on the virtual desktop.
    pub position: [f32; 2],
    /// Size of the monitor.
    pub size: [f32; 2],
    /// Working position (excluding taskbar/dock).
    pub work_pos: [f32; 2],
    /// Working size (excluding taskbar/dock).
    pub work_size: [f32; 2],
    /// DPI scale factor.
    pub dpi_scale: f32,
}

impl Default for MonitorData {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [1920.0, 1080.0],
            work_pos: [0.0, 0.0],
            work_size: [1920.0, 1080.0],
            dpi_scale: 1.0,
        }
    }
}

/// Initialize monitors in the platform IO.
///
/// This function populates the monitors list in PlatformIO, which is required
/// for viewport support to work correctly. Call this after enabling viewports
/// and before the first new_frame().
///
/// # Arguments
///
/// * `ctx` - The imgui context
/// * `monitors` - Monitor data to populate
///
/// # Example
///
/// ```rust,no_run
/// # use imgui::*;
/// # let mut ctx = Context::create();
/// # ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
/// monitor_init_fix::init_monitors(&mut ctx, &[
///     MonitorData::default(), // Primary monitor
/// ]);
/// ```
#[cfg(feature = "docking")]
pub fn init_monitors(_ctx: &mut Context, monitors: &[MonitorData]) {
    
    // Get platform IO
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        if io_ptr.is_null() {
            return;
        }
        &mut *(io_ptr as *mut crate::PlatformIo)
    };
    
    // Convert monitor data to PlatformMonitor format
    let platform_monitors: Vec<PlatformMonitor> = monitors
        .iter()
        .map(|data| PlatformMonitor {
            main_pos: data.position,
            main_size: data.size,
            work_pos: data.work_pos,
            work_size: data.work_size,
            dpi_scale: data.dpi_scale,
            platform_handle: std::ptr::null_mut(),
        })
        .collect();
    
    // Replace monitors in ImVector
    // Note: ImVector might have issues with empty slices, but we still call it
    // to ensure the vector is properly initialized
    platform_io.monitors.replace_from_slice(&platform_monitors);
}

/// Initialize monitors from a single primary monitor.
///
/// This is a convenience function for simple cases where you only have
/// one monitor or want to start with basic support.
///
/// # Arguments
///
/// * `ctx` - The imgui context
/// * `width` - Monitor width
/// * `height` - Monitor height
/// * `dpi_scale` - DPI scale factor (typically 1.0 for standard DPI)
#[cfg(feature = "docking")]
pub fn init_single_monitor(ctx: &mut Context, width: f32, height: f32, dpi_scale: f32) {
    let monitor = MonitorData {
        position: [0.0, 0.0],
        size: [width, height],
        work_pos: [0.0, 0.0],
        work_size: [width, height],
        dpi_scale,
    };
    
    init_monitors(ctx, &[monitor]);
}

/// Clear all monitors from the platform IO.
///
/// This can be useful for cleanup or resetting the monitor configuration.
#[cfg(feature = "docking")]
pub fn clear_monitors(ctx: &mut Context) {
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        if io_ptr.is_null() {
            return;
        }
        &mut *(io_ptr as *mut crate::PlatformIo)
    };
    
    platform_io.monitors.replace_from_slice(&[]);
}

/// Check if monitors are initialized.
///
/// Returns true if at least one monitor is present in the platform IO.
#[cfg(feature = "docking")]
pub fn monitors_initialized(_ctx: &Context) -> bool {
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        if io_ptr.is_null() {
            return false;
        }
        &*(io_ptr as *const crate::PlatformIo)
    };
    
    // Check if monitors are initialized by looking at the size field
    // We can't use as_slice() here as it might have a null pointer
    unsafe {
        let monitors_ptr = &platform_io.monitors as *const _ as *const sys::ImVector_ImGuiPlatformMonitor;
        (*monitors_ptr).Size > 0
    }
}

/// Get the count of initialized monitors.
#[cfg(feature = "docking")]
pub fn monitor_count(_ctx: &Context) -> usize {
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        if io_ptr.is_null() {
            return 0;
        }
        &*(io_ptr as *const crate::PlatformIo)
    };
    
    // Get count from the size field directly
    // We can't use as_slice() here as it might have a null pointer
    unsafe {
        let monitors_ptr = &platform_io.monitors as *const _ as *const sys::ImVector_ImGuiPlatformMonitor;
        (*monitors_ptr).Size as usize
    }
}

#[cfg(not(feature = "docking"))]
pub fn init_monitors(_ctx: &mut Context, _monitors: &[MonitorData]) {
    // No-op when docking feature is not enabled
}

#[cfg(not(feature = "docking"))]
pub fn init_single_monitor(_ctx: &mut Context, _width: f32, _height: f32, _dpi_scale: f32) {
    // No-op when docking feature is not enabled
}

#[cfg(not(feature = "docking"))]
pub fn clear_monitors(_ctx: &mut Context) {
    // No-op when docking feature is not enabled
}

#[cfg(not(feature = "docking"))]
pub fn monitors_initialized(_ctx: &Context) -> bool {
    false
}

#[cfg(not(feature = "docking"))]
pub fn monitor_count(_ctx: &Context) -> usize {
    0
}