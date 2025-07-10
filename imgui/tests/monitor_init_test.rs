//! Unit tests for monitor initialization fix

use parking_lot::Mutex;
use std::sync::LazyLock;

// Global mutex to prevent concurrent context creation
static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
#[cfg(feature = "docking")]
fn test_monitor_initialization() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    // Create context
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    
    // Initially, no monitors should be initialized
    assert!(!monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 0);
    
    // Initialize a single monitor
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    
    // Verify monitor was initialized
    assert!(monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 1);
    
    // Verify monitor data through platform IO
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        &*(io_ptr as *const PlatformIo)
    };
    
    // Access monitors carefully to avoid null pointer issues
    unsafe {
        let monitors_ptr = &platform_io.monitors as *const _ as *const sys::ImVector_ImGuiPlatformMonitor;
        assert_eq!((*monitors_ptr).Size, 1);
        
        // Only access data if size > 0
        if (*monitors_ptr).Size > 0 {
            let monitors = std::slice::from_raw_parts((*monitors_ptr).Data as *const PlatformMonitor, (*monitors_ptr).Size as usize);
            assert_eq!(monitors[0].main_pos, [0.0, 0.0]);
            assert_eq!(monitors[0].main_size, [1920.0, 1080.0]);
            assert_eq!(monitors[0].dpi_scale, 1.0);
        }
    }
}

#[test]
#[cfg(feature = "docking")]
fn test_multi_monitor_initialization() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    
    // Initialize multiple monitors
    let monitor_data = vec![
        monitor_init_fix::MonitorData {
            position: [0.0, 0.0],
            size: [1920.0, 1080.0],
            work_pos: [0.0, 0.0],
            work_size: [1920.0, 1040.0],
            dpi_scale: 1.0,
        },
        monitor_init_fix::MonitorData {
            position: [1920.0, 0.0],
            size: [2560.0, 1440.0],
            work_pos: [1920.0, 0.0],
            work_size: [2560.0, 1400.0],
            dpi_scale: 1.5,
        },
    ];
    
    monitor_init_fix::init_monitors(&mut ctx, &monitor_data);
    
    // Verify monitors were initialized
    assert!(monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 2);
    
    // Verify monitor data
    let platform_io = unsafe {
        let io_ptr = sys::igGetPlatformIO();
        &*(io_ptr as *const PlatformIo)
    };
    
    // Access monitors carefully to avoid null pointer issues
    unsafe {
        let monitors_ptr = &platform_io.monitors as *const _ as *const sys::ImVector_ImGuiPlatformMonitor;
        assert_eq!((*monitors_ptr).Size, 2);
        
        // Only access data if size > 0
        if (*monitors_ptr).Size > 0 {
            let monitors = std::slice::from_raw_parts((*monitors_ptr).Data as *const PlatformMonitor, (*monitors_ptr).Size as usize);
            
            // Check first monitor
            assert_eq!(monitors[0].main_pos, [0.0, 0.0]);
            assert_eq!(monitors[0].main_size, [1920.0, 1080.0]);
            assert_eq!(monitors[0].work_size, [1920.0, 1040.0]);
            assert_eq!(monitors[0].dpi_scale, 1.0);
            
            // Check second monitor
            assert_eq!(monitors[1].main_pos, [1920.0, 0.0]);
            assert_eq!(monitors[1].main_size, [2560.0, 1440.0]);
            assert_eq!(monitors[1].work_size, [2560.0, 1400.0]);
            assert_eq!(monitors[1].dpi_scale, 1.5);
        }
    }
}

#[test]
#[cfg(feature = "docking")]
fn test_clear_monitors() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    
    // Initialize monitors
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    assert!(monitor_init_fix::monitors_initialized(&ctx));
    
    // Clear monitors
    monitor_init_fix::clear_monitors(&mut ctx);
    
    // Verify monitors were cleared
    assert!(!monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 0);
}

#[test]
#[cfg(feature = "docking")]
fn test_monitor_data_default() {
    use imgui::monitor_init_fix::MonitorData;
    
    let data = MonitorData::default();
    assert_eq!(data.position, [0.0, 0.0]);
    assert_eq!(data.size, [1920.0, 1080.0]);
    assert_eq!(data.work_pos, [0.0, 0.0]);
    assert_eq!(data.work_size, [1920.0, 1080.0]);
    assert_eq!(data.dpi_scale, 1.0);
}

#[test]
#[cfg(feature = "docking")]
fn test_no_monitors_without_viewport_flag() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    let mut ctx = Context::create();
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    // Don't set VIEWPORTS_ENABLE
    
    // This should still work but is basically a no-op
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    
    // Since viewports aren't enabled, monitors might not be checked
    // This test verifies the functions don't crash when viewports are disabled
}

#[test]
#[cfg(not(feature = "docking"))]
fn test_monitor_functions_without_docking() {
    use imgui::*;
    
    let _lock = TEST_MUTEX.lock();
    
    let mut ctx = Context::create();
    
    // All functions should be no-ops when docking is disabled
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    monitor_init_fix::clear_monitors(&mut ctx);
    
    assert!(!monitor_init_fix::monitors_initialized(&ctx));
    assert_eq!(monitor_init_fix::monitor_count(&ctx), 0);
}