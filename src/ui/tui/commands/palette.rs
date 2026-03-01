use super::*;

impl UiCommand {
    pub(super) fn palette_spec(self) -> Option<PaletteCommandSpec> {
        self.meta().palette
    }
}
