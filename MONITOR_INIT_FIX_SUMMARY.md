# Monitor Initialization Fix Summary

## Issue Identified

When viewport support is enabled in imgui-rs, Dear ImGui throws an assertion error:
```
"Platform init didn't setup Monitors list?"
```

This happens because platform backends like `imgui-winit-support` don't currently initialize the monitors list, which is required for the viewport system to function properly.

## Root Cause

1. **Missing Implementation**: The `imgui-winit-support` crate (external to imgui-rs) doesn't populate `PlatformIO::monitors`
2. **Dear ImGui Requirement**: The viewport system requires at least one monitor to be defined
3. **Timing Critical**: Monitors must be initialized BEFORE any viewport operations

## Solution Implemented

Since `imgui-winit-support` is an external crate, we've created a workaround module `monitor_init_fix` that allows users to manually initialize monitors.

### Usage

```rust
use imgui::*;

// Create and configure context
let mut ctx = Context::create();
ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;

// Apply viewport fix (from previous issue)
viewport_issue_fix::apply_viewport_fix(&mut ctx);

// Initialize monitors BEFORE first new_frame()
monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);

// Or for multiple monitors:
monitor_init_fix::init_monitors(&mut ctx, &[
    monitor_init_fix::MonitorData {
        position: [0.0, 0.0],
        size: [1920.0, 1080.0],
        work_pos: [0.0, 0.0],
        work_size: [1920.0, 1040.0], // Excluding taskbar
        dpi_scale: 1.0,
    },
    monitor_init_fix::MonitorData {
        position: [1920.0, 0.0],
        size: [1920.0, 1080.0],
        work_pos: [1920.0, 0.0],
        work_size: [1920.0, 1080.0],
        dpi_scale: 1.0,
    },
]);
```

## Implementation Details

### Module Structure
- `monitor_init_fix.rs` - Main implementation
- Uses `PlatformIO::monitors` ImVector
- Provides convenience functions for common cases

### Key Functions
1. `init_monitors()` - Initialize with custom monitor data
2. `init_single_monitor()` - Quick single monitor setup
3. `clear_monitors()` - Clear all monitors
4. `monitors_initialized()` - Check if monitors are set
5. `monitor_count()` - Get number of monitors

### Integration with Winit

If you're using winit, you can enumerate real monitors:

```rust
use winit::event_loop::EventLoop;
use winit::dpi::{PhysicalPosition, PhysicalSize};

let event_loop = EventLoop::new();
let monitors: Vec<monitor_init_fix::MonitorData> = event_loop
    .available_monitors()
    .map(|monitor| {
        let PhysicalPosition { x, y } = monitor.position();
        let PhysicalSize { width, height } = monitor.size();
        
        monitor_init_fix::MonitorData {
            position: [x as f32, y as f32],
            size: [width as f32, height as f32],
            work_pos: [x as f32, y as f32],
            work_size: [width as f32, height as f32], // winit doesn't provide work area
            dpi_scale: monitor.scale_factor() as f32,
        }
    })
    .collect();

monitor_init_fix::init_monitors(&mut ctx, &monitors);
```

## Files Added/Modified

### New Files
- `imgui/src/monitor_init_fix.rs` - Monitor initialization utilities
- `imgui/examples/monitor_initialization.rs` - Usage example
- `imgui/tests/monitor_init_test.rs` - Unit tests
- `imgui/tests/monitor_viewport_integration_test.rs` - Integration tests

### Modified Files
- `imgui/src/lib.rs` - Added `pub mod monitor_init_fix`

## Known Limitations

1. **Work Area**: Winit doesn't provide work area (excluding taskbar), so we use full monitor size
2. **Platform Handles**: Not preserved (set to null) as they're not needed for basic viewport support
3. **Hot-plug Support**: Monitor changes require manual re-initialization

## Complete Example

```rust
use imgui::*;

fn main() {
    let mut ctx = Context::create();
    
    // Configure for viewports
    ctx.io_mut().config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
    ctx.io_mut().display_size = [1280.0, 720.0];
    ctx.io_mut().delta_time = 1.0 / 60.0;
    ctx.fonts().build_rgba32_texture();
    
    // Apply fixes
    viewport_issue_fix::apply_viewport_fix(&mut ctx);
    monitor_init_fix::init_single_monitor(&mut ctx, 1920.0, 1080.0, 1.0);
    
    // Set platform backend
    ctx.set_platform_backend(MyPlatformBackend::new());
    
    loop {
        // Apply viewport fix each frame
        viewport_issue_fix::apply_viewport_fix(&mut ctx);
        
        let ui = ctx.new_frame();
        
        // Windows can now be dragged outside!
        ui.window("Draggable")
            .size([300.0, 200.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Drag me outside!");
            });
        
        let draw_data = ctx.render();
        // ... render draw_data ...
        
        ctx.update_platform_windows();
        ctx.render_platform_windows_default();
    }
}
```

## Testing

Run the tests with:
```bash
cargo test --features docking monitor
```

Run the example:
```bash
cargo run --example monitor_initialization --features docking
```

## Future Improvements

1. **Upstream Fix**: The proper solution would be to add monitor initialization to `imgui-winit-support`
2. **Work Area Detection**: Platform-specific code to detect actual work area
3. **Auto-initialization**: Integrate with platform backend setup
4. **Monitor Events**: Handle display configuration changes automatically

## Summary

This workaround successfully enables viewport support by manually initializing the monitors list. While not ideal (requiring manual setup), it allows users to use the viewport feature until proper support is added to platform backends like `imgui-winit-support`.