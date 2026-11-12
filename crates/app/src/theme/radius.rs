use epub_core::domain::settings::CornerRadiusStyle;
use gpui_kit::{Pixels, px};

pub trait RadiusTokens {
    fn base_radius(&self) -> Pixels;
    fn sm_radius(&self) -> Pixels;
    fn lg_radius(&self) -> Pixels;
    fn full_radius(&self) -> Pixels;
}

impl RadiusTokens for CornerRadiusStyle {
    fn base_radius(&self) -> Pixels {
        match self {
            CornerRadiusStyle::Square => px(0.0),
            CornerRadiusStyle::Subtle => px(4.0),
            CornerRadiusStyle::Rounded => px(8.0),
            CornerRadiusStyle::Circular => px(9999.0),
        }
    }

    fn sm_radius(&self) -> Pixels {
        match self {
            CornerRadiusStyle::Square => px(0.0),
            CornerRadiusStyle::Subtle => px(2.0),
            CornerRadiusStyle::Rounded => px(4.0),
            CornerRadiusStyle::Circular => px(9999.0),
        }
    }

    fn lg_radius(&self) -> Pixels {
        match self {
            CornerRadiusStyle::Square => px(0.0),
            CornerRadiusStyle::Subtle => px(6.0),
            CornerRadiusStyle::Rounded => px(12.0),
            CornerRadiusStyle::Circular => px(9999.0),
        }
    }

    fn full_radius(&self) -> Pixels {
        px(9999.0)
    }
}
