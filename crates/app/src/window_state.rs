use std::path::PathBuf;
use gpui_kit::{point, px, size, Bounds, WindowBounds};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWindowState {
    pub is_maximized: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

fn window_state_path() -> Option<PathBuf> {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        Some(PathBuf::from(config_home).join("epub-reader").join("window_state.json"))
    } else if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".config").join("epub-reader").join("window_state.json"))
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        Some(PathBuf::from(appdata).join("epub-reader").join("window_state.json"))
    } else {
        None
    }
}

impl SavedWindowState {
    pub fn load() -> Option<Self> {
        let path = window_state_path()?;
        let data = std::fs::read_to_string(path).ok()?;
        let state: Self = serde_json::from_str(&data).ok()?;
        // Sanity checks: reject nonsensical geometry
        if state.width < 400.0 || state.height < 300.0 {
            return None;
        }
        Some(state)
    }

    pub fn save(&self) {
        if let Some(path) = window_state_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn to_window_bounds(&self) -> WindowBounds {
        let origin = point(px(self.x), px(self.y));
        let size = size(px(self.width.max(800.0)), px(self.height.max(600.0)));
        let bounds = Bounds { origin, size };
        if self.is_maximized {
            WindowBounds::Maximized(bounds)
        } else {
            WindowBounds::Windowed(bounds)
        }
    }

    pub fn from_window_bounds(bounds: WindowBounds) -> Self {
        match bounds {
            WindowBounds::Maximized(b) => Self {
                is_maximized: true,
                x: b.origin.x.as_f32(),
                y: b.origin.y.as_f32(),
                width: b.size.width.as_f32(),
                height: b.size.height.as_f32(),
            },
            WindowBounds::Windowed(b) => Self {
                is_maximized: false,
                x: b.origin.x.as_f32(),
                y: b.origin.y.as_f32(),
                width: b.size.width.as_f32(),
                height: b.size.height.as_f32(),
            },
            WindowBounds::Fullscreen(b) => Self {
                is_maximized: true,
                x: b.origin.x.as_f32(),
                y: b.origin.y.as_f32(),
                width: b.size.width.as_f32(),
                height: b.size.height.as_f32(),
            },
        }
    }
}
