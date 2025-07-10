//! Test for viewport fix

#[test]
#[cfg(feature = "docking")]
fn test_viewport_fix_works() {
    use imgui::*;
    
    // Create context without using test helper to avoid premature new_frame
    let mut ctx = Context::create();
    
    // Initialize IO
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    
    // Apply fix BEFORE any frames
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Set platform backend
    struct TestBackend;
    impl PlatformViewportBackend for TestBackend {
        fn create_window(&mut self, _: &mut Viewport) {
            println!("create_window called!");
        }
        fn destroy_window(&mut self, _: &mut Viewport) {}
        fn show_window(&mut self, _: &mut Viewport) {}
        fn set_window_pos(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_pos(&mut self, _: &mut Viewport) -> [f32; 2] { [0.0, 0.0] }
        fn set_window_size(&mut self, _: &mut Viewport, _: [f32; 2]) {}
        fn get_window_size(&mut self, _: &mut Viewport) -> [f32; 2] { [0.0, 0.0] }
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
    
    ctx.set_platform_backend(TestBackend);
    
    // Build fonts to avoid issues
    ctx.fonts().build_rgba32_texture();
    
    // First frame
    assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE));
    let _ui = ctx.new_frame();
    let _dd = ctx.render();
    
    // Without fix, flags would be reset here
    // With fix, we reapply them
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    
    // Second frame - flags should still be set
    assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE));
    let _ui = ctx.new_frame();
    let _dd = ctx.render();
    
    // Verify fix continues to work
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    assert!(ctx.io().config_flags.contains(ConfigFlags::VIEWPORTS_ENABLE));
    
    println!("Viewport fix test passed!");
}