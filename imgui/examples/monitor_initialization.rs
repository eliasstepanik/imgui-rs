//! Example demonstrating how to fix monitor initialization for viewport support.
//!
//! This example shows how to properly initialize monitors to avoid the
//! "Platform init didn't setup Monitors list?" assertion error.

use imgui::*;
#[cfg(feature = "docking")]
use imgui::monitor_init_fix;

// Mock window system for demonstration
struct MockWindow {
    width: f32,
    height: f32,
}

impl MockWindow {
    fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    
    // Mock function to simulate getting available monitors
    fn available_monitors(&self) -> Vec<MockMonitor> {
        vec![
            MockMonitor {
                position: [0.0, 0.0],
                size: [1920.0, 1080.0],
                scale_factor: 1.0,
            },
            MockMonitor {
                position: [1920.0, 0.0],
                size: [1920.0, 1080.0],
                scale_factor: 1.0,
            },
        ]
    }
}

struct MockMonitor {
    position: [f32; 2],
    size: [f32; 2],
    scale_factor: f32,
}

fn main() {
    println!("=== MONITOR INITIALIZATION EXAMPLE ===");
    println!("This demonstrates how to fix the monitor initialization issue.\n");
    
    #[cfg(feature = "docking")]
    {
        // Create context
        let mut ctx = Context::create();
        
        // Initialize IO
        ctx.io_mut().display_size = [1280.0, 720.0];
        ctx.io_mut().delta_time = 1.0 / 60.0;
        
        // Build fonts
        ctx.fonts().build_rgba32_texture();
        
        // Enable viewport support
        ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
        
        // Apply viewport fix (from previous issue)
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        // CRITICAL: Initialize monitors before first new_frame()
        println!("Initializing monitors...");
        
        // Method 1: Simple single monitor initialization
        monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
        println!("✓ Initialized single monitor: 1920x1080 @ 1.0x DPI");
        
        // Verify monitors are initialized
        if monitor_init_fix::monitors_initialized(&ctx) {
            println!("✓ Monitors initialized successfully!");
            println!("  Monitor count: {}", monitor_init_fix::monitor_count(&ctx));
        } else {
            println!("✗ Monitors not initialized!");
        }
        
        println!("\nClearing monitors and trying multi-monitor setup...");
        monitor_init_fix::clear_monitors(&mut ctx);
        
        // Method 2: Multi-monitor initialization with real data
        let window = MockWindow::new(1280.0, 720.0);
        let monitors = window.available_monitors();
        
        // Convert mock monitors to MonitorData
        let monitor_data: Vec<monitor_init_fix::MonitorData> = monitors
            .iter()
            .map(|m| monitor_init_fix::MonitorData {
                position: m.position,
                size: m.size,
                work_pos: m.position,
                work_size: [m.size[0], m.size[1] - 40.0], // Simulate taskbar
                dpi_scale: m.scale_factor,
            })
            .collect();
        
        // Initialize monitors
        monitor_init_fix::init_monitors(&mut ctx, &monitor_data);
        println!("✓ Initialized {} monitors", monitor_data.len());
        
        for (i, data) in monitor_data.iter().enumerate() {
            println!("  Monitor {}: pos={:?}, size={:?}, dpi={}", 
                i, data.position, data.size, data.dpi_scale);
        }
        
        // Set up a mock platform backend
        struct TestPlatformBackend {
            viewports_created: u32,
        }
        
        impl PlatformViewportBackend for TestPlatformBackend {
            fn create_window(&mut self, viewport: &mut Viewport) {
                self.viewports_created += 1;
                println!("\n🎉 VIEWPORT CREATED! Total: {}", self.viewports_created);
                println!("   ID: {:?}", viewport.id);
                println!("   Position: {:?}", viewport.pos);
                println!("   Size: {:?}", viewport.size);
                viewport.platform_window_created = true;
            }
            
            fn destroy_window(&mut self, _viewport: &mut Viewport) {}
            fn show_window(&mut self, _viewport: &mut Viewport) {}
            fn set_window_pos(&mut self, _viewport: &mut Viewport, _pos: [f32; 2]) {}
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
            fn set_window_title(&mut self, _viewport: &mut Viewport, _title: &str) {}
            fn set_window_alpha(&mut self, _viewport: &mut Viewport, _alpha: f32) {}
            fn update_window(&mut self, _viewport: &mut Viewport) {}
            fn render_window(&mut self, _viewport: &mut Viewport) {}
            fn swap_buffers(&mut self, _viewport: &mut Viewport) {}
            fn create_vk_surface(&mut self, _: &mut Viewport, _: u64, _: &mut u64) -> i32 { 0 }
        }
        
        let backend = TestPlatformBackend {
            viewports_created: 0,
        };
        ctx.set_platform_backend(backend);
        
        println!("\nTesting viewport creation with initialized monitors...");
        
        // Simulate a few frames
        for frame in 0..5 {
            println!("\n─── Frame {} ───", frame);
            
            // Apply viewport fix before each frame
            viewport_issue_fix::apply_viewport_fix(&mut ctx);
            
            // Get monitor count before creating frame
            let monitor_count = monitor_init_fix::monitor_count(&ctx);
            
            let ui = ctx.new_frame();
            
            // Main window
            ui.window("Monitor Init Demo")
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([50.0, 50.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("✅ Monitors initialized!");
                    ui.text(format!("Monitor count: {}", monitor_count));
                    ui.separator();
                    ui.text("Drag windows outside to test viewport creation.");
                });
            
            // Window that moves outside on frame 3
            if frame >= 3 {
                ui.window("External Window")
                    .size([300.0, 150.0], Condition::FirstUseEver)
                    .position([1500.0, 100.0], Condition::Always)
                    .build(|| {
                        ui.text("This window is outside!");
                        ui.text("Should trigger viewport creation.");
                    });
            }
            
            // Render - this should NOT trigger assertion error
            let _draw_data = ctx.render();
            
            // Update platform windows
            ctx.update_platform_windows();
            
            // Show viewport stats
            let viewport_count = ctx.viewports().count();
            println!("Active viewports: {}", viewport_count);
            
            ctx.render_platform_windows_default();
        }
        
        println!("\n\n✅ SUCCESS!");
        println!("Monitor initialization fix is working correctly.");
        println!("No assertion errors were triggered.");
        println!("\nTo use this fix in your code:");
        println!("1. Enable viewports: ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE");
        println!("2. Initialize monitors: monitor_init_fix::init_monitors(&mut ctx, &monitors)");
        println!("3. Apply viewport fix before each frame (if needed)");
    }
    
    #[cfg(not(feature = "docking"))]
    {
        println!("This example requires the 'docking' feature.");
        println!("Run with: cargo run --example monitor_initialization --features docking");
    }
}