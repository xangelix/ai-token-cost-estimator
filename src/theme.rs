//! Light / dark theme management.

#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl Theme {
    pub const fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        };
    }

    pub fn apply(self, ctx: &egui::Context) {
        let visuals = match self {
            Self::Light => egui::Visuals::light(),
            Self::Dark => egui::Visuals::dark(),
        };
        ctx.set_visuals(visuals);
    }

    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }
}
