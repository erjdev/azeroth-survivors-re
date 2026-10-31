use super::*;

#[derive(Component, Default)]
pub(in crate::survivors) struct HemorrhageVulnerability(pub survivors_core::hemorrhage::Hemorrhage);

impl HemorrhageVulnerability {
    pub(super) fn bonus(
        &mut self,
        source: &str,
        origin: HitOrigin,
        damage: f32,
        now: f32,
        spells: Option<&crate::ui_action::Spells>,
        reflection: u32,
    ) -> f32 {
        if self.0.charges(now) == 0 {
            return 0.0;
        }
        let physical = if (origin == HitOrigin::Hero
            && matches!(source, "sword_specialization" | "Weapon Attack"))
            || (origin == HitOrigin::CombatPet && source == crate::survivors::hunter_pet::SOURCE)
        {
            true // Generated weapon strikes and authored pet bites have no ability row.
        } else {
            let id = match source {
                "Deep Wounds" => Some(12721),
                "Counterattack" => Some(19306),
                "Thorns" => Some(reflection),
                "instant_poison" => Some(8680),
                "deadly_poison" => Some(2818),
                _ => data::ABILITIES
                    .iter()
                    .find(|d| d.key == source)
                    .map(|d| d.spell),
            };
            id.and_then(|id| spells.and_then(|s| s.catalog.get(id)))
                .is_some_and(|s| s.school == 0)
        };
        self.0.bonus(physical, damage, now)
    }
}
