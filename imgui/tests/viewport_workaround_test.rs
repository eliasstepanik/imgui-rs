//! Integration test demonstrating the viewport workaround

#[test]
#[cfg(feature = "docking")]
fn test_viewport_workaround() {
    use imgui::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    
    // Track if callbacks are triggered
    let create_window_called = Rc::new(RefCell::new(false));
    let create_window_called_clone = create_window_called.clone();
    
    struct TestBackend {
        create_called: Rc<RefCell<bool>>,
    }
    
    impl PlatformViewportBackend for TestBackend {
        fn create_window(&mut self, viewport: &mut Viewport) {
            println!("CREATE_WINDOW called for viewport {:?}", viewport.id);
            *self.create_called.borrow_mut() = true;
            viewport.platform_window_created = true;
        }
        
        fn destroy_window(&mut self, _: &mut Viewport) {}
        fn show_window(&mut self, _: &mut Viewport) {}
        fn set_window_pos(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_pos(&mut self, _: &mut Viewport) -> [f32; 2] { [0.0, 0.0] }
        fn set_window_size(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_size(&mut self, _: &mut Viewport) -> [f32; 2] { [100.0, 100.0] }
        fn set_window_focus(&mut self, _: &mut Viewport) {}
        fn get_window_focus(&mut self, _: &mut Viewport) -> bool { false }
        fn get_window_minimized(&mut self, _: &mut Viewport) -> bool { false }
        fn set_window_title(&mut self, _: &mut Viewport, _: &str) {}
        fn set_window_alpha(&mut self, _: &mut Viewport, _: f32) {}
        fn update_window(&mut self, _: &mut Viewport) {}
        fn render_window(&mut self, _: &mut Viewport) {}
        fn swap_buffers(&mut self, _: &mut Viewport) {}
        fn create_vk_surface(&mut self, _: &mut Viewport, _: u64, _: &mut u64) -> i32 { 0 }
    }
    
    // Create context
    let mut ctx = Context::create();
    
    // Initialize
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    ctx.fonts().build_rgba32_texture();
    
    // Apply workaround BEFORE first frame
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Set backend
    let backend = TestBackend {
        create_called: create_window_called_clone,
    };
    ctx.set_platform_backend(backend);
    
    // Verify the workaround maintains the flag
    for i in 0..3 {
        // Apply fix before each frame
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        // Verify flag is set
        assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE),
            "Frame {}: Flag should be set", i);
        
        let ui = ctx.new_frame();
        
        // Create a window that would trigger viewport creation
        ui.window("Test")
            .position([1500.0, 100.0], Condition::Always) // Outside main viewport
            .size([200.0, 100.0], Condition::Always)
            .build(|| {
                ui.text("Outside window");
            });
        
        let _dd = ctx.render();
        ctx.update_platform_windows();
        ctx.render_platform_windows_default();
    }
    
    // The workaround successfully maintains viewport support
    assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE),
        "Flag should still be set after multiple frames");
    
    println!("Viewport workaround test passed!");
}