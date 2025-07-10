//! Example demonstrating multi-viewport rendering support in imgui-rs.
//!
//! This example shows how to:
//! - Enable viewport support
//! - Iterate over all viewports
//! - Safely render each viewport's DrawData
//! - Implement platform and renderer viewport backends

use imgui::*;
use std::collections::HashMap;

/// Example renderer that can handle multiple viewports
struct MyRenderer {
    // In a real implementation, you'd store per-viewport rendering state here
    viewport_renderers: HashMap<Id, ViewportRenderer>,
}

struct ViewportRenderer {
    // Per-viewport rendering resources (e.g., framebuffers, textures)
    _id: Id,
}

impl MyRenderer {
    fn new() -> Self {
        Self {
            viewport_renderers: HashMap::new(),
        }
    }

    /// Render the main viewport (backward compatible method)
    fn render_main(&mut self, draw_data: &DrawData) {
        println!("Rendering main viewport");
        // In a real implementation, you'd render the draw data here
        self.render_draw_data(draw_data);
    }

    /// Render a specific viewport
    fn render_viewport(&mut self, viewport_id: Id, draw_data: &DrawData) {
        println!("Rendering viewport: {:?}", viewport_id);
        
        // Get or create renderer for this viewport
        let _renderer = self.viewport_renderers
            .entry(viewport_id)
            .or_insert_with(|| ViewportRenderer { _id: viewport_id });
        
        // In a real implementation, you'd render the draw data here
        self.render_draw_data(draw_data);
    }

    fn render_draw_data(&self, draw_data: &DrawData) {
        println!(
            "  DrawData: {} draw lists, {} vertices, {} indices",
            draw_data.draw_lists_count(),
            draw_data.total_vtx_count,
            draw_data.total_idx_count
        );
    }
}

/// Example platform backend implementation
#[cfg(feature = "docking")]
struct MyPlatformBackend {
    // Platform-specific window handles would be stored here
    windows: HashMap<Id, PlatformWindow>,
}

#[cfg(feature = "docking")]
struct PlatformWindow {
    _id: Id,
    // In a real implementation, this would be an OS window handle
}

#[cfg(feature = "docking")]
impl PlatformViewportBackend for MyPlatformBackend {
    fn create_window(&mut self, viewport: &mut Viewport) {
        println!("Creating window for viewport: {:?}", viewport.id);
        self.windows.insert(
            viewport.id,
            PlatformWindow { _id: viewport.id }
        );
    }

    fn destroy_window(&mut self, viewport: &mut Viewport) {
        println!("Destroying window for viewport: {:?}", viewport.id);
        self.windows.remove(&viewport.id);
    }

    fn show_window(&mut self, viewport: &mut Viewport) {
        println!("Showing window for viewport: {:?}", viewport.id);
    }

    fn set_window_pos(&mut self, viewport: &mut Viewport, pos: [f32; 2]) {
        println!("Setting window position for viewport {:?}: {:?}", viewport.id, pos);
    }

    fn get_window_pos(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.pos
    }

    fn set_window_size(&mut self, viewport: &mut Viewport, size: [f32; 2]) {
        println!("Setting window size for viewport {:?}: {:?}", viewport.id, size);
    }

    fn get_window_size(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.size
    }

    fn set_window_focus(&mut self, viewport: &mut Viewport) {
        println!("Setting focus to viewport: {:?}", viewport.id);
    }

    fn get_window_focus(&mut self, _viewport: &mut Viewport) -> bool {
        // In a real implementation, query OS for window focus
        true
    }

    fn get_window_minimized(&mut self, _viewport: &mut Viewport) -> bool {
        // In a real implementation, query OS for window state
        false
    }

    fn set_window_title(&mut self, viewport: &mut Viewport, title: &str) {
        println!("Setting window title for viewport {:?}: {}", viewport.id, title);
    }

    fn set_window_alpha(&mut self, viewport: &mut Viewport, alpha: f32) {
        println!("Setting window alpha for viewport {:?}: {}", viewport.id, alpha);
    }

    fn update_window(&mut self, viewport: &mut Viewport) {
        // Update platform window state
        println!("Updating window for viewport: {:?}", viewport.id);
    }

    fn render_window(&mut self, viewport: &mut Viewport) {
        // Platform-specific rendering setup
        println!("Platform render for viewport: {:?}", viewport.id);
    }

    fn swap_buffers(&mut self, viewport: &mut Viewport) {
        // Swap buffers for the viewport's window
        println!("Swapping buffers for viewport: {:?}", viewport.id);
    }

    fn create_vk_surface(
        &mut self,
        _viewport: &mut Viewport,
        _instance: u64,
        _out_surface: &mut u64,
    ) -> i32 {
        // For Vulkan backends
        0
    }
}

/// Main rendering function that handles all viewports
#[cfg(feature = "docking")]
fn render_all_viewports(ctx: &mut Context, renderer: &mut MyRenderer) {
    // Method 1: Render main viewport using backward-compatible API
    let main_draw_data = ctx.render();
    renderer.render_main(main_draw_data);
    
    // Update platform windows (creates/destroys OS windows as needed)
    // Must be called AFTER render()
    ctx.update_platform_windows();
    
    // Method 2: Iterate over all viewports
    for viewport in ctx.viewports() {
        // Skip main viewport as we already rendered it
        if viewport.is_main() {
            continue;
        }
        
        // Only render if viewport has content
        if let Some(draw_data) = viewport.draw_data() {
            renderer.render_viewport(viewport.id, draw_data);
        } else {
            println!("Viewport {:?} has no content to render", viewport.id);
        }
    }
    
    // Let imgui handle platform window presentation
    ctx.render_platform_windows_default();
}

/// Alternative approach: render all viewports uniformly
#[cfg(feature = "docking")]
fn render_all_viewports_uniform(ctx: &mut Context, renderer: &mut MyRenderer) {
    // First, we need to call render() to generate draw data
    let _main_draw_data = ctx.render();
    
    // Update platform windows (must be called AFTER render())
    ctx.update_platform_windows();
    
    // Now iterate over ALL viewports including main
    for viewport in ctx.viewports() {
        if let Some(draw_data) = viewport.draw_data() {
            if viewport.is_main() {
                println!("Rendering main viewport uniformly");
                renderer.render_main(draw_data);
            } else {
                renderer.render_viewport(viewport.id, draw_data);
            }
        }
    }
    
    ctx.render_platform_windows_default();
}

/// Example showing error handling for viewport rendering
#[cfg(feature = "docking")]
fn render_with_error_handling(ctx: &mut Context, renderer: &mut MyRenderer) -> Result<(), String> {
    let main_draw_data = ctx.render();
    renderer.render_main(main_draw_data);
    
    // Update platform windows (must be called AFTER render())
    ctx.update_platform_windows();
    
    for viewport in ctx.viewports() {
        if viewport.is_main() {
            continue;
        }
        
        match viewport.draw_data() {
            Some(draw_data) => {
                renderer.render_viewport(viewport.id, draw_data);
            }
            None => {
                // This is not an error - viewport might just have no visible content
                println!("Viewport {:?} has no draw data (might be hidden)", viewport.id);
            }
        }
    }
    
    ctx.render_platform_windows_default();
    Ok(())
}

fn main() {
    println!("Multi-viewport rendering example");
    
    #[cfg(feature = "docking")]
    {
        // Create imgui context
        let mut ctx = Context::create();
        
        // Initialize IO properly for new_frame()
        let io = ctx.io_mut();
        io.display_size = [1280.0, 720.0];
        io.delta_time = 1.0 / 60.0;
        
        // Enable viewports before first frame
        ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
        
        // Build font atlas
        ctx.fonts().build_rgba32_texture();
        
        // Set up platform backend
        let platform_backend = MyPlatformBackend {
            windows: HashMap::new(),
        };
        ctx.set_platform_backend(platform_backend);
        
        // Create renderer
        let mut renderer = MyRenderer::new();
        
        // Simulate a frame
        println!("\n--- Frame 1: Basic viewport rendering ---");
        ctx.new_frame();
        // ... build UI here ...
        render_all_viewports(&mut ctx, &mut renderer);
        
        println!("\n--- Frame 2: Uniform viewport rendering ---");
        ctx.new_frame();
        // ... build UI here ...
        render_all_viewports_uniform(&mut ctx, &mut renderer);
        
        println!("\n--- Frame 3: With error handling ---");
        ctx.new_frame();
        // ... build UI here ...
        if let Err(e) = render_with_error_handling(&mut ctx, &mut renderer) {
            eprintln!("Rendering error: {}", e);
        }
    }
    
    #[cfg(not(feature = "docking"))]
    {
        println!("This example requires the 'docking' feature to be enabled.");
        println!("Run with: cargo run --example viewport_rendering --features docking");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "docking")]
    fn test_viewport_helpers() {
        let mut ctx = Context::create();
        
        // Initialize IO properly
        let io = ctx.io_mut();
        io.display_size = [1280.0, 720.0];
        io.delta_time = 1.0 / 60.0;
        ctx.fonts().build_rgba32_texture();
        
        // Must be within a frame to access viewport data
        ctx.new_frame();
        
        // Test main viewport
        let main_viewport = ctx.main_viewport();
        assert!(main_viewport.is_main());
        
        // Test viewport iteration
        let viewport_count = ctx.viewports().count();
        assert!(viewport_count >= 1); // At least main viewport exists
    }

    #[test]
    #[cfg(feature = "docking")]
    fn test_null_draw_data_safety() {
        let mut ctx = Context::create();
        
        // Initialize IO properly
        let io = ctx.io_mut();
        io.display_size = [1280.0, 720.0];
        io.delta_time = 1.0 / 60.0;
        ctx.fonts().build_rgba32_texture();
        
        // Must be within a frame to access viewport data
        ctx.new_frame();
        
        // Without rendering, viewports might not have draw data
        for viewport in ctx.viewports() {
            // This should not panic even if draw_data is null
            let _draw_data = viewport.draw_data();
        }
    }
}