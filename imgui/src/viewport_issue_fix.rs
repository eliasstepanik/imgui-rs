//! Fix for viewport window creation issue in imgui-rs
//!
//! ## The Issue
//! 
//! ConfigFlags::VIEWPORTS_ENABLE is being reset after the first render() call.
//! This prevents viewport callbacks (create_window, destroy_window) from being
//! triggered when windows are dragged outside the main window.
//!
//! ## Root Cause
//!
//! The imgui-rs Context accesses IO via sys::igGetIO() which returns a pointer
//! to the C++ ImGuiIO structure. Something in the render pipeline is clearing
//! the ConfigFlags, causing viewport support to be disabled after the first frame.
//!
//! ## The Fix
//!
//! We need to ensure ConfigFlags::VIEWPORTS_ENABLE is set:
//! 1. BEFORE the first new_frame() call (required by Dear ImGui)
//! 2. Restored before EACH subsequent new_frame() call
//!
//! This module provides utilities to work around this issue until it's fixed
//! in imgui-rs or the underlying Dear ImGui library.

use crate::{sys, ConfigFlags, Context};

/// Apply the viewport fix by ensuring flags persist across frames
///
/// Call this function:
/// 1. After creating the Context
/// 2. Before EACH new_frame() call
///
/// # Example
/// ```no_run
/// # use imgui::*;
/// # let mut ctx = unsafe { Context::create_internal(None) };
/// // Initial setup
/// ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
/// apply_viewport_fix(&mut ctx);
/// 
/// // In render loop - call before EACH frame
/// loop {
///     apply_viewport_fix(&mut ctx);
///     let ui = ctx.new_frame();
///     // ... render UI ...
/// }
/// ```
pub fn apply_viewport_fix(ctx: &mut Context) {
    // Ensure the flag is set in both Rust and C++ sides
    ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    
    unsafe {
        let io_ptr = sys::igGetIO();
        if !io_ptr.is_null() {
            (*io_ptr).ConfigFlags |= sys::ImGuiConfigFlags_ViewportsEnable as i32;
        }
    }
}

/// Check if the viewport issue is affecting this context
///
/// Returns true if viewports were enabled but have been reset
pub fn is_viewport_issue_present(ctx: &Context) -> bool {
    unsafe {
        let io_ptr = sys::igGetIO();
        if !io_ptr.is_null() {
            let cpp_flags = (*io_ptr).ConfigFlags;
            let viewport_bit = sys::ImGuiConfigFlags_ViewportsEnable as i32;
            
            // Check if the flag is NOT set in C++ but IS expected in Rust
            let cpp_has_flag = (cpp_flags & viewport_bit) != 0;
            let rust_expects_flag = ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE);
            
            // Issue is present if Rust expects it but C++ doesn't have it
            rust_expects_flag && !cpp_has_flag
        } else {
            false
        }
    }
}

/// Initialize viewport support with automatic fix application
///
/// Returns a closure that should be called before each new_frame()
pub fn initialize_viewport_support_with_fix(ctx: &mut Context) -> impl FnMut(&mut Context) {
    // Enable viewports initially
    apply_viewport_fix(ctx);
    
    // Return a closure that applies the fix
    move |ctx: &mut Context| {
        apply_viewport_fix(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(feature = "docking")]
    fn test_viewport_fix() {
        let (_guard, mut ctx) = crate::test::test_ctx_initialized();
        
        // Apply the fix
        apply_viewport_fix(&mut ctx);
        
        // Check it's set
        assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE));
        
        // Check C++ side
        unsafe {
            let io_ptr = sys::igGetIO();
            assert!(!io_ptr.is_null());
            let cpp_flags = (*io_ptr).ConfigFlags;
            let viewport_bit = sys::ImGuiConfigFlags_ViewportsEnable as i32;
            assert_ne!(cpp_flags & viewport_bit, 0);
        }
    }
}