//! Debug example for viewport window creation issue
//! This example helps debug why viewport callbacks aren't triggered

use imgui::*;
use std::collections::HashMap;
use std::cell::RefCell;

/// Debug platform backend that logs all calls
#[cfg(feature = "docking")]
struct DebugPlatformBackend {
    windows: HashMap<Id, DebugWindow>,
    call_log: RefCell<Vec<String>>,
}

#[cfg(feature = "docking")]
struct DebugWindow {
    id: Id,
    created_at: std::time::Instant,
}

#[cfg(feature = "docking")]
impl DebugPlatformBackend {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            call_log: RefCell::new(Vec::new()),
        }
    }
    
    fn log(&self, msg: String) {
        println!("[PLATFORM] {}", msg);
        self.call_log.borrow_mut().push(msg);
    }
}

#[cfg(feature = "docking")]
impl PlatformViewportBackend for DebugPlatformBackend {
    fn create_window(&mut self, viewport: &mut Viewport) {
        self.log(format!("=== CREATE_WINDOW CALLED ==="));
        self.log(format!("  Viewport ID: {:?}", viewport.id));
        self.log(format!("  Flags: {:?}", viewport.flags));
        self.log(format!("  Position: {:?}", viewport.pos));
        self.log(format!("  Size: {:?}", viewport.size));
        self.log(format!("  DPI Scale: {}", viewport.dpi_scale));
        self.log(format!("  platform_window_created: {}", viewport.platform_window_created));
        
        self.windows.insert(
            viewport.id,
            DebugWindow { 
                id: viewport.id,
                created_at: std::time::Instant::now(),
            }
        );
        
        // Mark that we created the platform window
        viewport.platform_window_created = true;
    }

    fn destroy_window(&mut self, viewport: &mut Viewport) {
        self.log(format!("=== DESTROY_WINDOW CALLED === ID: {:?}", viewport.id));
        self.windows.remove(&viewport.id);
    }

    fn show_window(&mut self, viewport: &mut Viewport) {
        self.log(format!("SHOW_WINDOW: {:?}", viewport.id));
    }

    fn set_window_pos(&mut self, viewport: &mut Viewport, pos: [f32; 2]) {
        self.log(format!("SET_WINDOW_POS: {:?} -> {:?}", viewport.id, pos));
    }

    fn get_window_pos(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.pos
    }

    fn set_window_size(&mut self, viewport: &mut Viewport, size: [f32; 2]) {
        self.log(format!("SET_WINDOW_SIZE: {:?} -> {:?}", viewport.id, size));
    }

    fn get_window_size(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        viewport.size
    }

    fn set_window_focus(&mut self, viewport: &mut Viewport) {
        self.log(format!("SET_WINDOW_FOCUS: {:?}", viewport.id));
    }

    fn get_window_focus(&mut self, viewport: &mut Viewport) -> bool {
        let focused = !viewport.is_main();
        self.log(format!("GET_WINDOW_FOCUS: {:?} = {}", viewport.id, focused));
        focused
    }

    fn get_window_minimized(&mut self, viewport: &mut Viewport) -> bool {
        self.log(format!("GET_WINDOW_MINIMIZED: {:?} = false", viewport.id));
        false
    }

    fn set_window_title(&mut self, viewport: &mut Viewport, title: &str) {
        self.log(format!("SET_WINDOW_TITLE: {:?} = '{}'", viewport.id, title));
    }

    fn set_window_alpha(&mut self, viewport: &mut Viewport, alpha: f32) {
        self.log(format!("SET_WINDOW_ALPHA: {:?} = {}", viewport.id, alpha));
    }

    fn update_window(&mut self, viewport: &mut Viewport) {
        self.log(format!("UPDATE_WINDOW: {:?}", viewport.id));
    }

    fn render_window(&mut self, viewport: &mut Viewport) {
        self.log(format!("RENDER_WINDOW: {:?}", viewport.id));
    }

    fn swap_buffers(&mut self, viewport: &mut Viewport) {
        self.log(format!("SWAP_BUFFERS: {:?}", viewport.id));
    }

    fn create_vk_surface(
        &mut self,
        viewport: &mut Viewport,
        _instance: u64,
        _out_surface: &mut u64,
    ) -> i32 {
        self.log(format!("CREATE_VK_SURFACE: {:?}", viewport.id));
        0
    }
}

fn main() {
    println!("=== VIEWPORT DEBUG EXAMPLE ===");
    println!("This example helps debug viewport window creation issues.");
    println!();
    
    #[cfg(feature = "docking")]
    {
        // Create imgui context
        let mut ctx = Context::create();
        
        // Initialize IO properly
        let io = ctx.io_mut();
        io.display_size = [1280.0, 720.0];
        io.delta_time = 1.0 / 60.0;
        
        println!("Initial ConfigFlags: {:?}", io.config_flags);
        
        // Enable viewports
        println!("ConfigFlags before: {:?}", ctx.io().config_flags);
        println!("Available flags: {:?}", ConfigFlags::all());
        println!("VIEWPORTS_ENABLE flag value: {:?}", ConfigFlags::VIEWPORTS_ENABLE);
        
        // Try using the fix function
        ensure_viewport_flags(&mut ctx);
        
        println!("ConfigFlags after ensure_viewport_flags: {:?}", ctx.io().config_flags);
        
        // Double check it's actually set
        if !ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
            println!("ERROR: VIEWPORTS_ENABLE flag not set!");
        } else {
            println!("SUCCESS: VIEWPORTS_ENABLE flag is set!");
        }
        
        // Check C++ side directly
        unsafe {
            let io_ptr = sys::igGetIO();
            if !io_ptr.is_null() {
                println!("C++ ConfigFlags value: 0x{:08X}", (*io_ptr).ConfigFlags);
                println!("VIEWPORTS_ENABLE constant: 0x{:08X}", sys::ImGuiConfigFlags_ViewportsEnable);
            }
        }
        
        // Build font atlas
        ctx.fonts().build_rgba32_texture();
        
        // Set up debug platform backend
        let platform_backend = DebugPlatformBackend::new();
        ctx.set_platform_backend(platform_backend);
        
        println!("\n=== STARTING FRAME SIMULATION ===\n");
        
        // Check flag right before frames
        println!("ConfigFlags before frame loop: {:?}", ctx.io().config_flags);
        
        // Simulate several frames
        for frame in 0..5 {
            println!("\n>>> FRAME {} <<<", frame);
            
            // Get main viewport info before new_frame
            let main_vp_id;
            let main_vp_pos;
            let main_vp_size;
            {
                let main_vp = ctx.main_viewport();
                main_vp_id = main_vp.id;
                main_vp_pos = main_vp.pos;
                main_vp_size = main_vp.size;
            }
            println!("Main viewport: ID={:?}, pos={:?}, size={:?}", 
                main_vp_id, main_vp_pos, main_vp_size);
            
            // Check flags right before new_frame
            println!("ConfigFlags RIGHT BEFORE new_frame: {:?}", ctx.io().config_flags);
            
            let ui = ctx.new_frame();
            
            // Window 1: Should be draggable outside
            ui.window("Test Window 1")
                .size([300.0, 200.0], Condition::FirstUseEver)
                .position([100.0, 100.0], Condition::FirstUseEver)
                .flags(WindowFlags::empty()) // No restrictive flags
                .build(|| {
                    ui.text("This window should be draggable outside");
                    ui.text("Drag me outside the main window!");
                    
                    let window_pos = ui.window_pos();
                    let window_size = ui.window_size();
                    ui.text(format!("Pos: {:?}", window_pos));
                    ui.text(format!("Size: {:?}", window_size));
                });
            
            // Window 2: Debug info window
            ui.window("Debug Info")
                .size([400.0, 300.0], Condition::FirstUseEver)
                .position([500.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Viewport Debug Information:");
                    ui.separator();
                    
                    ui.text(format!("Frame: {}", frame));
                    
                    // We'll check viewport info after render
                    ui.text("See console for viewport details");
                });
            
            // Simulate window being dragged outside on frame 2
            if frame == 2 {
                println!("\n!!! SIMULATING WINDOW DRAG OUTSIDE !!!");
                // In a real scenario, the user would drag the window
                // Here we're just changing position programmatically
                ui.window("Forced Outside Window")
                    .position([1400.0, 100.0], Condition::Always)
                    .size([200.0, 100.0], Condition::FirstUseEver)
                    .flags(WindowFlags::empty())
                    .build(|| {
                        ui.text("I'm outside!");
                    });
            }
            
            // Render and update viewports
            println!("\nCalling render()...");
            let draw_data = ctx.render();
            println!("Main viewport draw data: {} vertices, {} indices",
                draw_data.total_vtx_count, draw_data.total_idx_count);
            
            // Now check viewport info after render
            println!("\nViewport info after render:");
            println!("ConfigFlags: {:?}", ctx.io().config_flags);
            if ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE) {
                println!("Viewports: ENABLED");
            } else {
                println!("Viewports: DISABLED");
            }
            
            // Check C++ side after render
            unsafe {
                let io_ptr = sys::igGetIO();
                if !io_ptr.is_null() {
                    println!("C++ ConfigFlags after render: 0x{:08X}", (*io_ptr).ConfigFlags);
                }
            }
            
            println!("\nPre-update viewport count: {}", ctx.viewports().count());
            
            println!("\nCalling update_platform_windows()...");
            ctx.update_platform_windows();
            
            println!("\nPost-update viewport count: {}", ctx.viewports().count());
            
            // Check all viewports after update
            for (i, viewport) in ctx.viewports().enumerate() {
                println!("  Viewport {}: ID={:?}, is_main={}, created={}", 
                    i, viewport.id, viewport.is_main(), viewport.platform_window_created);
                
                if let Some(dd) = viewport.draw_data() {
                    println!("    Has draw data: {} vertices", dd.total_vtx_count);
                } else {
                    println!("    No draw data");
                }
            }
            
            println!("\nCalling render_platform_windows_default()...");
            ctx.render_platform_windows_default();
        }
        
        println!("\n=== SIMULATION COMPLETE ===");
    }
    
    #[cfg(not(feature = "docking"))]
    {
        println!("This example requires the 'docking' feature to be enabled.");
        println!("Run with: cargo run --example viewport_debug --features docking");
    }
}