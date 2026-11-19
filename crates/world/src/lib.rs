//! Campaign rules outside battle: the map and its fog of war, story flags,
//! dialogues, encounters, shops, experience and levels (`PLAN.md` §2 and §4).
//!
//! Like the engine, this crate does no I/O and keeps no clock: the server
//! loads the content into a [`World`], keeps a [`Game`] per player, feeds it
//! [`GameCommand`]s, runs the battles it asks for with the engine, and
//! stores its [`Progress`] and [`Save`].
//!
//! | Module | Contents |
//! | --- | --- |
//! | [`def`] | The world as content describes it |
//! | [`game`] | A game in progress: commands, scenes, battles and rewards |
//! | [`progression`] | Levels, experience, hero stats |
//! | [`save`] | What is stored between sessions |
//! | [`validate`] | Checks that content references exist |
//! | [`view`] | What the player sees (sent to the client) |

pub mod def;
pub mod game;
mod ids;
pub mod progression;
pub mod save;
#[cfg(test)]
mod test_world;
pub mod validate;
pub mod view;

use std::collections::BTreeMap;

use engine::{ActionDef, ActionId, AiAct, CombatantSetup, ItemDef, ItemId, Stats};

pub use def::WorldDef;
pub use game::{BattleOrder, Game, GameCommand, GameError, ShopGood};
pub use ids::{
    ClassId, DialogueId, EquipmentId, Flag, HeroId, InstanceId, MonsterId, NodeId, Position,
    RegionId, ShopId,
};
pub use save::{GameMode, HeroState, Progress, Save, SavedBattle};

/// Everything the rules read: the world, plus the action and item
/// definitions battles need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    /// Regions, instances, heroes, monsters, dialogues, shops, equipment.
    pub map: WorldDef,
    /// Every action, by id.
    pub actions: BTreeMap<ActionId, ActionDef>,
    /// Every item, by id.
    pub items: BTreeMap<ItemId, ItemDef>,
}

impl World {
    /// Battle stats of a hero in its current level and gear; `None` when
    /// the content does not know the hero or its class.
    #[must_use]
    pub fn hero_stats(&self, hero: &HeroState) -> Option<Stats> {
        let class = self
            .map
            .classes
            .get(&self.map.heroes.get(&hero.hero)?.class)?;
        let piece =
            |id: &Option<EquipmentId>| id.as_ref().and_then(|id| self.map.equipment.get(id));
        let gear = progression::equipment_of(piece(&hero.weapon), piece(&hero.armor));
        Some(progression::hero_stats(class, hero.level, gear))
    }

    /// A hero as it enters a battle, carrying its HP and MP.
    #[must_use]
    pub fn hero_setup(&self, hero: &HeroState) -> Option<CombatantSetup> {
        let class = self
            .map
            .classes
            .get(&self.map.heroes.get(&hero.hero)?.class)?;
        let stats = self.hero_stats(hero)?;
        let actions = progression::known_actions(class, hero.level);
        // The class AI covers every level; a hero uses the rules it can.
        let mut ai = class.ai.clone();
        for phase in &mut ai.phases {
            phase.rules.retain(|rule| match &rule.act {
                AiAct::Action { id } => actions.contains(id),
                AiAct::Attack | AiAct::Item { .. } => true,
            });
        }
        Some(CombatantSetup {
            name: format!("hero-{}", hero.hero),
            attack: class.attack.clone(),
            actions,
            affinities: BTreeMap::new(),
            status_immunities: Vec::new(),
            ai: Some(ai),
            hp: Some(hero.hp.clamp(1, stats.max_hp)),
            mp: Some(hero.mp.min(stats.max_mp)),
            stats,
            guest: hero.guest,
            leader: false,
        })
    }

    /// A monster as it enters a battle.
    #[must_use]
    pub fn monster_setup(&self, monster: &MonsterId) -> Option<CombatantSetup> {
        Some(self.map.monsters.get(monster)?.battle.clone())
    }

    /// The creatures `enemies` can raise a fallen ally into
    /// ([`engine::Effect::Reanimate`] names a monster id), and those the
    /// raised ones can raise in turn.
    #[must_use]
    pub fn forms_for(&self, enemies: &[CombatantSetup]) -> BTreeMap<String, CombatantSetup> {
        let mut forms = BTreeMap::new();
        let mut pending: Vec<CombatantSetup> = enemies.to_vec();
        while let Some(setup) = pending.pop() {
            for action in setup.actions.iter().filter_map(|id| self.actions.get(id)) {
                if let engine::Effect::Reanimate { into } = &action.effect
                    && !forms.contains_key(into)
                    && let Some(form) = self.monster_setup(&MonsterId::new(into))
                {
                    pending.push(form.clone());
                    forms.insert(into.clone(), form);
                }
            }
        }
        forms
    }
}
