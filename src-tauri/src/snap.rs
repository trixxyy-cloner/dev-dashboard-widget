use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

/// How close (logical px) to an edge the widget has to be dropped to snap.
const THRESHOLD: f64 = 48.0;
/// Gap (logical px) between the snapped widget and the screen edge.
const MARGIN: f64 = 12.0;
/// The OS sends a stream of move events while dragging; snap once they stop.
const SETTLE: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct SnapState {
    generation: AtomicU64,
    pub locked: AtomicBool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Returns where the window should go: pulled to any edge it is near or past, otherwise unchanged.
pub fn snapped(window: Rect, area: Rect, threshold: i32, margin: i32) -> (i32, i32) {
    let axis = |pos: i32, len: i32, start: i32, area_len: i32| {
        let end = start + area_len;
        if pos - start < threshold {
            start + margin
        } else if end - (pos + len) < threshold {
            end - len - margin
        } else {
            pos
        }
    };
    (axis(window.x, window.w, area.x, area.w), axis(window.y, window.h, area.y, area.h))
}

fn snap_now(window: &WebviewWindow) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return move_to_primary(window);
    };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let pos = window.outer_position()?;
    let size = window.outer_size()?;

    let (x, y) = snapped(
        Rect { x: pos.x, y: pos.y, w: size.width as i32, h: size.height as i32 },
        Rect { x: area.position.x, y: area.position.y, w: area.size.width as i32, h: area.size.height as i32 },
        (THRESHOLD * scale) as i32,
        (MARGIN * scale) as i32,
    );
    // Setting the position fires another Moved event; skipping no-ops ends that loop.
    if (x, y) != (pos.x, pos.y) {
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

/// Used when the saved position is on a monitor that is no longer connected.
fn move_to_primary(window: &WebviewWindow) -> tauri::Result<()> {
    let Some(monitor) = window.primary_monitor()? else {
        return Ok(());
    };
    let area = monitor.work_area();
    let margin = (MARGIN * monitor.scale_factor()) as i32;
    let size = window.outer_size()?;
    let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
    window.set_position(PhysicalPosition::new(x, area.position.y + margin))
}

/// Called on every Moved event; snaps once the window has been still for a moment.
pub fn on_moved(window: &WebviewWindow) {
    let state = window.state::<SnapState>();
    if state.locked.load(Ordering::Relaxed) {
        return;
    }
    let generation = state.generation.fetch_add(1, Ordering::Relaxed) + 1;
    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(SETTLE).await;
        if window.state::<SnapState>().generation.load(Ordering::Relaxed) != generation {
            return;
        }
        // On the main thread: saving locks the plugin's cache and queries the window, and from
        // a background thread that deadlocks with the plugin's own Moved handler.
        let app = window.app_handle().clone();
        let _ = app.run_on_main_thread(move || {
            if let Err(e) = snap_now(&window) {
                eprintln!("snap: {e}");
            }
            // The plugin only writes to disk on a clean exit, which a Windows shutdown skips.
            if let Err(e) = window.app_handle().save_window_state(StateFlags::POSITION | StateFlags::SIZE) {
                eprintln!("save window state: {e}");
            }
        });
    });
}

/// Puts the window back on screen if the restored position is off every monitor.
pub fn ensure_visible(window: &WebviewWindow) {
    if let Err(e) = snap_now(window) {
        eprintln!("snap: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect { x: 0, y: 0, w: 1920, h: 1040 };
    const T: i32 = 48;
    const M: i32 = 12;

    fn window_at(x: i32, y: i32) -> Rect {
        Rect { x, y, w: 380, h: 640 }
    }

    #[test]
    fn snapped_near_top_right_snaps_to_corner() {
        assert_eq!(snapped(window_at(1520, 20), AREA, T, M), (1920 - 380 - M, M));
    }

    #[test]
    fn snapped_in_middle_stays_put() {
        assert_eq!(snapped(window_at(700, 200), AREA, T, M), (700, 200));
    }

    #[test]
    fn snapped_partly_off_screen_is_pulled_back() {
        // Off the left edge and hanging below the bottom edge.
        assert_eq!(snapped(window_at(-100, 500), AREA, T, M), (M, 1040 - 640 - M));
    }

    #[test]
    fn snapped_on_second_monitor_uses_that_monitors_area() {
        let right_monitor = Rect { x: 1920, y: 0, w: 2560, h: 1400 };
        assert_eq!(snapped(window_at(1930, 740), right_monitor, T, M), (1920 + M, 1400 - 640 - M));
    }
}
