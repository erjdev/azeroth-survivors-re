//! Read-only appearance access for the local Survivors body compositor.
impl super::Creatures {
    pub(crate) fn survivors_appearance(
        &self,
        display: u32,
    ) -> Option<benilla_formats::NpcAppearance> {
        self.catalog.model(display)?.npc_appearance
    }
}
