pub mod palettes;
pub mod radius;
pub mod state;
pub mod tokens;
pub mod typography;

pub use palettes::*;
pub use radius::*;
pub use state::*;
pub use tokens::*;
pub use typography::*;

use gpui_kit::App;

pub fn init_theme(cx: &mut App) {
    let state = ThemeState::default();
    state.sync_with_gpui(cx);
    cx.set_global(state);
}
