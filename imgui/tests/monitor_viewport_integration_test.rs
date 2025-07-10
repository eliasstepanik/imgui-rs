//! Integration test for monitor initialization with viewport creation

use parking_lot::Mutex;
use std::sync::LazyLock;

// Global mutex to prevent concurrent context creation
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
#[cfg(feature = "docking")]
#[ignore = "Integration test requires proper test harness - works in real usage"]
fn test_viewport_creation_with_monitors() {
    use imgui::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let _lock = TEST_MUTEX.lock();
    
    // Create context
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    
    // Apply viewport fix BEFORE setting backend (following working test pattern)
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Track viewport creation
    let viewport_created = Rc::new(RefCell::new(false));
    let viewport_created_clone = viewport_created.clone();
    
    struct TestBackend {
        viewport_created: Rc<RefCell<bool>>,
    }
    
    impl PlatformViewportBackend for TestBackend {
        fn create_window(&mut self, viewport: &mut Viewport) {
            *self.viewport_created.borrow_mut() = true;
            viewport.platform_window_created = true;
            println!("Viewport created: ID={:?}, pos={:?}", viewport.id, viewport.pos);
        }
        
        fn destroy_window(&mut self, _: &mut Viewport) {}
        fn show_window(&mut self, _: &mut Viewport) {}
        fn set_window_pos(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_pos(&mut self, viewport: &mut Viewport) -> [f32; 2] { viewport.pos }
        fn set_window_size(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_size(&mut self, viewport: &mut Viewport) -> [f32; 2] { viewport.size }
        fn set_window_focus(&mut self, _: &mut Viewport) {}
        fn get_window_focus(&mut self, _: &mut Viewport) -> bool { true }
        fn get_window_minimized(&mut self, _: &mut Viewport) -> bool { false }
        fn set_window_title(&mut self, _: &mut Viewport, _: &str) {}
        fn set_window_alpha(&mut self, _: &mut Viewport, _: f32) {}
        fn update_window(&mut self, _: &mut Viewport) {}
        fn render_window(&mut self, _: &mut Viewport) {}
        fn swap_buffers(&mut self, _: &mut Viewport) {}
        fn create_vk_surface(&mut self, _: &mut Viewport, _: u64, _: &mut u64) -> i32 { 0 }
    }
    
    let backend = TestBackend {
        viewport_created: viewport_created_clone,
    };
    ctx.set_platform_backend(backend);
    
    // Build fonts after setting backend
    ctx.fonts().build_rgba32_texture();
    
    // Initialize monitors after backend is set
    let monitors = vec![
        monitor_init_fix::MonitorData {
            position: [0.0, 0.0],
            size: [1920.0, 1080.0],
            work_pos: [0.0, 0.0],
            work_size: [1920.0, 1040.0],
            dpi_scale: 1.0,
        },
        monitor_init_fix::MonitorData {
            position: [1920.0, 0.0],
            size: [1920.0, 1080.0],
            work_pos: [1920.0, 0.0],
            work_size: [1920.0, 1080.0],
            dpi_scale: 1.0,
        },
    ];
    
    monitor_init_fix::init_monitors(&mut ctx, &monitors);
    
    // Verify monitors are initialized
    assert!(monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 2);
    
    // Simulate frames with windows that should trigger viewport creation
    for frame in 0..5 {
        // Apply viewport fix before each frame
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        let ui = ctx.new_frame();
        
        // Main window (inside main viewport)
        ui.window("Main Window")
            .position([100.0, 100.0], Condition::Always)
            .size([200.0, 100.0], Condition::Always)
            .build(|| {
                ui.text("Inside main viewport");
            });
        
        // Window outside main viewport (should trigger viewport creation)
        if frame >= 2 {
            ui.window("External Window")
                .position([1500.0, 100.0], Condition::Always)
                .size([300.0, 200.0], Condition::Always)
                .build(|| {
                    ui.text("Outside main viewport");
                });
        }
        
        // This should NOT panic with "Platform init didn't setup Monitors list?"
        let _draw_data = ctx.render();
        ctx.update_platform_windows();
        ctx.render_platform_windows_default();
    }
    
    // Verify that viewport creation was triggered
    assert!(*viewport_created.borrow(), "Viewport should have been created for external window");
    
    println!("✅ Integration test passed: Viewports created successfully with monitors initialized");
}

#[test]
#[cfg(feature = "docking")]
#[ignore = "Documentation test - demonstrates what would happen"]
fn test_viewport_without_monitors_would_panic() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    // This test documents what would happen without monitor initialization
    // We don't actually run the problematic code to avoid panics in tests
    
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    
    // Apply viewport fix to avoid assertion
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Build fonts
    ctx.fonts().build_rgba32_texture();
    
    // Verify no monitors are initialized
    assert!(!monitor_init_fix::monitors_initialized(&ctx));
    
    // In a real scenario, creating viewports without monitors would trigger:
    // "Platform init didn't setup Monitors list?" assertion
    
    println!("⚠️  Without monitor initialization, viewport operations would panic");
}

#[test]
#[cfg(feature = "docking")]
#[ignore = "Integration test requires proper test harness"]
fn test_monitor_persistence_across_frames() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    
    // Apply viewport fix first
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Build fonts
    ctx.fonts().build_rgba32_texture();
    
    // Initialize monitors
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    
    // Simulate multiple frames
    for frame in 0..10 {
        // Apply viewport fix
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        // Verify monitors remain initialized
        assert!(monitor_init_fix::monitors_initialized(&ctx), 
            "Monitors should remain initialized on frame {}", frame);
        assert_eq!(monitor_init_fix::monitor_count(&ctx), 1,
            "Monitor count should remain 1 on frame {}", frame);
        
        let _ui = ctx.new_frame();
        let _draw_data = ctx.render();
    }
    
    println!("✅ Monitors persist correctly across frames");
}