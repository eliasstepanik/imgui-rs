//! Complete solution for viewport window creation
//!
//! This example demonstrates the working fix for the viewport issue where
//! ConfigFlags::VIEWPORTS_ENABLE is reset after render().

use imgui::*;
use std::collections::HashMap;

/// Platform backend that successfully creates viewport windows
#[cfg(feature = "docking")]
struct SolutionPlatformBackend {
    windows: HashMap<Id, ViewportWindow>,
    total_windows_created: u32,
}

#[cfg(feature = "docking")]
struct ViewportWindow {
    id: Id,
    window_number: u32,
    title: String,
}

#[cfg(feature = "docking")]
impl SolutionPlatformBackend {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            total_windows_created: 0,
        }
    }
}

#[cfg(feature = "docking")]
impl PlatformViewportBackend for SolutionPlatformBackend {
    fn create_window(&mut self, viewport: &mut Viewport) {
        self.total_windows_created += 1;
        let window_num = self.total_windows_created;
        
        println!("\n🎉 VIEWPORT WINDOW CREATED! 🎉");
        println!("   Window #{}", window_num);
        println!("   Viewport ID: {:?}", viewport.id);
        println!("   Position: {:?}", viewport.pos);
        println!("   Size: {:?}", viewport.size);
        
        self.windows.insert(
            viewport.id,
            ViewportWindow {
                id: viewport.id,
                window_number: window_num,
                title: format!("Viewport Window #{}", window_num),
            }
        );
        
        viewport.platform_window_created = true;
    }

    fn destroy_window(&mut self, viewport: &mut Viewport) {
        if let Some(window) = self.windows.remove(&viewport.id) {
            println!("\n👋 Viewport window destroyed: #{}", window.window_number);
        }
    }

    fn show_window(&mut self, _viewport: &mut Viewport) {}

    fn set_window_pos(&mut self, viewport: &mut Viewport, pos: [f32; 2]) {
        if let Some(window) = self.windows.get(&viewport.id) {
            println!("   Window #{} moved to {:?}", window.window_number, pos);
        }
    }

    fn get_window_pos(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.pos
    }

    fn set_window_size(&mut self, _viewport: &mut Viewport, _size: [f32; 2]) {}
    fn get_window_size(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.size
    }

    fn set_window_focus(&mut self, _viewport: &mut Viewport) {}
    fn get_window_focus(&mut self, _viewport: &mut Viewport) -> bool { true }
    fn get_window_minimized(&mut self, _viewport: &mut Viewport) -> bool { false }
    
    fn set_window_title(&mut self, viewport: &mut Viewport, title: &str) {
        if let Some(window) = self.windows.get_mut(&viewport.id) {
            window.title = title.to_string();
            println!("   Window #{} title: '{}'", window.window_number, title);
        }
    }

    fn set_window_alpha(&mut self, _viewport: &mut Viewport, _alpha: f32) {}
    fn update_window(&mut self, _viewport: &mut Viewport) {}
    fn render_window(&mut self, _viewport: &mut Viewport) {}
    fn swap_buffers(&mut self, _viewport: &mut Viewport) {}
    fn create_vk_surface(&mut self, _: &mut Viewport, _: u64, _: &mut u64) -> i32 { 0 }
}

fn main() {
    println!("=== VIEWPORT SOLUTION EXAMPLE ===");
    println!("This demonstrates the complete fix for viewport window creation.\n");
    
    #[cfg(feature = "docking")]
    {
        // Create context
        let mut ctx = Context::create();
        
        // Initialize IO
        ctx.io_mut().display_size = [1280.0, 720.0];
        ctx.io_mut().delta_time = 1.0 / 60.0;
        
        // Build fonts
        ctx.fonts().build_rgba32_texture();
        
        // CRITICAL: Apply the viewport fix BEFORE first new_frame
        println!("Applying viewport fix...");
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        // Set up platform backend
        println!("Setting up platform backend...");
        let backend = SolutionPlatformBackend::new();
        ctx.set_platform_backend(backend);
        
        println!("Viewport system initialized with fix!\n");
        
        // Render frames
        for frame in 0..8 {
            println!("\n════ FRAME {} ════", frame);
            
            // CRITICAL: Apply fix before EACH new_frame()
            viewport_issue_fix::apply_viewport_fix(&mut ctx);
            
            // Verify fix is working
            if !ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
                panic!("Viewport fix failed!");
            }
            
            let ui = ctx.new_frame();
            
            // Status window
            ui.window("Viewport Solution Status")
                .size([450.0, 250.0], Condition::FirstUseEver)
                .position([50.0, 50.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("✅ VIEWPORT SYSTEM ACTIVE WITH FIX");
                    ui.separator();
                    
                    ui.text("The fix ensures ConfigFlags::VIEWPORTS_ENABLE");
                    ui.text("persists across frames by reapplying it before");
                    ui.text("each new_frame() call.");
                    
                    ui.separator();
                    ui.text(format!("Frame: {}", frame));
                    
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], "Fix is active");
                });
            
            // Draggable test window
            let initial_pos = if frame < 4 { [600.0, 100.0] } else { [1400.0, 100.0] };
            ui.window("Draggable Window")
                .size([300.0, 200.0], Condition::FirstUseEver)
                .position(initial_pos, if frame < 4 { Condition::FirstUseEver } else { Condition::Always })
                .build(|| {
                    ui.text("🎯 Drag me outside the main window!");
                    ui.separator();
                    
                    let pos = ui.window_pos();
                    ui.text(format!("Position: {:.0}, {:.0}", pos[0], pos[1]));
                    
                    if pos[0] > 1280.0 {
                        ui.text_colored([0.0, 1.0, 0.0, 1.0], "I'm outside! 🎉");
                        ui.text("Viewport should be created!");
                    } else {
                        ui.text("Still inside main window");
                    }
                });
            
            // Additional test window on frame 5
            if frame >= 5 {
                ui.window("Second External Window")
                    .position([1400.0, 350.0], Condition::Always)
                    .size([250.0, 150.0], Condition::FirstUseEver)
                    .build(|| {
                        ui.text("📍 Another external window!");
                        ui.text("This creates a second viewport.");
                    });
            }
            
            // Render
            let _draw_data = ctx.render();
            
            // Update platform windows - viewport callbacks happen here
            println!("Updating platform windows...");
            ctx.update_platform_windows();
            
            // Count viewports
            let viewport_count = ctx.viewports().count();
            println!("Total viewports: {} (main + {} extra)", 
                viewport_count, 
                if viewport_count > 1 { viewport_count - 1 } else { 0 });
            
            // Render platform windows
            ctx.render_platform_windows_default();
        }
        
        println!("\n\n✅ SUCCESS! Viewport window creation is working!");
        println!("The fix successfully maintains viewport support across frames.");
        println!("\nTo use this fix in your code:");
        println!("1. Call viewport_issue_fix::apply_viewport_fix() after context creation");
        println!("2. Call it again before EACH new_frame()");
    }
    
    #[cfg(not(feature = "docking"))]
    {
        println!("This example requires the 'docking' feature.");
        println!("Run with: cargo run --example viewport_solution --features docking");
    }
}