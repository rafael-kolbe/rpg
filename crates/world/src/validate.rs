//! Checks of `content/world/` that JSON parsing cannot do: every reference
//! points at something that exists, monsters and heroes can actually enter a
//! battle, and a battle effect is always the last of its list. Run by
//! `content-check` (and so by `make check`).

use engine::{Battle, BattleSetup, Encounter, ItemId, Opening, PartyControl};

use crate::World;
use crate::def::{NodeKind, QuestGoal, StoryEffect};
use crate::ids::{DialogueId, EquipmentId, HeroId, MonsterId, Position, QuestId};
use crate::progression::MAX_LEVEL;
use crate::save::HeroState;

/// Most enemies in one fight (the battle screen's room).
const MAX_ENEMIES: usize = 4;

/// Every problem found, as readable sentences; empty when the world is sound.
#[must_use]
pub fn problems(world: &World) -> Vec<String> {
    let mut check = Checker {
        world,
        problems: Vec::new(),
    };
    check.start();
    check.heroes();
    check.monsters();
    check.instances();
    check.dialogues();
    check.shops();
    check.equipment();
    check.quests();
    check.problems
}

/// Fluent message ids the world needs in every locale.
#[must_use]
pub fn message_ids(world: &World) -> Vec<String> {
    let map = &world.map;
    let mut ids = Vec::new();
    ids.extend(map.regions.keys().map(|id| format!("region-{id}")));
    ids.extend(map.instances.keys().map(|id| format!("instance-{id}")));
    ids.extend(map.classes.keys().map(|id| format!("class-{id}")));
    ids.extend(map.heroes.keys().map(|id| format!("hero-{id}")));
    ids.extend(map.monsters.keys().map(|id| format!("monster-{id}")));
    ids.extend(map.equipment.keys().map(|id| format!("equipment-{id}")));
    ids.extend(map.quests.keys().map(|id| format!("quest-{id}")));
    for dialogue in map.dialogues.values() {
        objective_ids(&dialogue.effects, &mut ids);
        for choice in &dialogue.choices {
            objective_ids(&choice.effects, &mut ids);
        }
    }
    for instance in map.instances.values() {
        for node in instance.nodes.values() {
            if let NodeKind::Battle { on_win, .. } = &node.kind {
                objective_ids(on_win, &mut ids);
            }
        }
    }
    for (id, dialogue) in &map.dialogues {
        for (n, speaker) in dialogue.lines.iter().enumerate() {
            ids.push(format!("dialogue-{id}-{}", n + 1));
            ids.extend(speaker.clone());
        }
        ids.extend((1..=dialogue.choices.len()).map(|n| format!("dialogue-{id}-choice-{n}")));
    }
    for instance in map.instances.values() {
        for node in instance.nodes.values() {
            ids.extend(node.label.clone());
            if let NodeKind::Npc { name, .. } = &node.kind {
                ids.push(name.clone());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// The objective messages effects show, including after their battles.
fn objective_ids(effects: &[StoryEffect], ids: &mut Vec<String>) {
    for effect in effects {
        match effect {
            StoryEffect::Objective {
                message: Some(message),
            } => ids.push(message.clone()),
            StoryEffect::Battle { on_win, .. } => objective_ids(on_win, ids),
            _ => {}
        }
    }
}

struct Checker<'a> {
    world: &'a World,
    problems: Vec<String>,
}

impl Checker<'_> {
    fn report(&mut self, problem: String) {
        self.problems.push(problem);
    }

    fn position(&mut self, owner: &str, position: &Position) {
        let exists = self
            .world
            .map
            .instances
            .get(&position.instance)
            .is_some_and(|instance| instance.nodes.contains_key(&position.node));
        if !exists {
            self.report(format!(
                "{owner}: no node `{}` in instance `{}`",
                position.node, position.instance
            ));
        }
    }

    fn item(&mut self, owner: &str, item: &ItemId) {
        if !self.world.items.contains_key(item) {
            self.report(format!("{owner}: unknown item `{item}`"));
        }
    }

    fn hero(&mut self, owner: &str, hero: &HeroId) {
        if !self.world.map.heroes.contains_key(hero) {
            self.report(format!("{owner}: unknown hero `{hero}`"));
        }
    }

    fn piece(&mut self, owner: &str, id: &EquipmentId) {
        if !self.world.map.equipment.contains_key(id) {
            self.report(format!("{owner}: unknown equipment `{id}`"));
        }
    }

    fn dialogue(&mut self, owner: &str, id: &DialogueId) {
        if !self.world.map.dialogues.contains_key(id) {
            self.report(format!("{owner}: unknown dialogue `{id}`"));
        }
    }

    fn quest(&mut self, owner: &str, quest: &QuestId) {
        if !self.world.map.quests.contains_key(quest) {
            self.report(format!("{owner}: unknown quest `{quest}`"));
        }
    }

    fn quests(&mut self) {
        for (id, quest) in &self.world.map.quests {
            let owner = format!("quest `{id}`");
            let QuestGoal::Kill { monsters, count } = &quest.goal;
            if *count == 0 {
                self.report(format!("{owner}: goal of 0"));
            }
            for monster in monsters {
                if !self.world.map.monsters.contains_key(monster) {
                    self.report(format!("{owner}: unknown monster `{monster}`"));
                }
            }
            for event in &quest.events {
                self.dialogue(&owner, &event.dialogue);
                if event.at == 0 || event.at > *count {
                    self.report(format!(
                        "{owner}: event at {} outside 1..={count}",
                        event.at
                    ));
                }
            }
        }
    }

    fn fight(&mut self, owner: &str, monsters: &[MonsterId]) {
        if monsters.is_empty() || monsters.len() > MAX_ENEMIES {
            self.report(format!("{owner}: a fight needs 1..={MAX_ENEMIES} monsters"));
        }
        for monster in monsters {
            if !self.world.map.monsters.contains_key(monster) {
                self.report(format!("{owner}: unknown monster `{monster}`"));
            }
        }
    }

    fn effects(&mut self, owner: &str, effects: &[StoryEffect]) {
        for (index, effect) in effects.iter().enumerate() {
            match effect {
                StoryEffect::SetFlag { .. }
                | StoryEffect::Kalds { .. }
                | StoryEffect::Heal
                | StoryEffect::Objective { .. }
                | StoryEffect::SetRank { .. }
                | StoryEffect::Autosave => {}
                StoryEffect::Join { hero, level, .. } => {
                    self.hero(owner, hero);
                    if level.is_some_and(|level| level == 0 || level > MAX_LEVEL) {
                        self.report(format!("{owner}: join level out of 1..=99"));
                    }
                }
                StoryEffect::Leave { hero } => self.hero(owner, hero),
                StoryEffect::StartQuest { quest } => self.quest(owner, quest),
                StoryEffect::Travel { to } => self.position(owner, to),
                StoryEffect::Scene { dialogue } => self.dialogue(owner, dialogue),
                StoryEffect::Give { item, .. } => self.item(owner, item),
                StoryEffect::GiveEquipment { id } => self.piece(owner, id),
                StoryEffect::Battle {
                    monsters, on_win, ..
                } => {
                    if index + 1 != effects.len() {
                        self.report(format!("{owner}: a battle effect must be the last"));
                    }
                    self.fight(owner, monsters);
                    self.effects(owner, on_win);
                }
            }
        }
    }

    /// Starts a battle to let the engine validate the fighters.
    fn can_fight(
        &mut self,
        owner: &str,
        party: Vec<engine::CombatantSetup>,
        enemy: engine::CombatantSetup,
    ) {
        let setup = BattleSetup {
            encounter: Encounter {
                opening: Opening::Normal,
                inventory: std::collections::BTreeMap::new(),
                forms: self.world.forms_for(std::slice::from_ref(&enemy)),
                party,
                enemies: vec![enemy],
            },
            actions: self.world.actions.clone(),
            items: self.world.items.clone(),
        };
        if let Err(error) = Battle::new(setup, 0, PartyControl::Ai) {
            self.report(format!("{owner}: {error}"));
        }
    }

    fn start(&mut self) {
        let start = &self.world.map.start;
        self.position("start", &start.position.clone());
        if start.party.is_empty() {
            self.report("start: the party is empty".to_owned());
        }
        for hero in &start.party.clone() {
            self.hero("start", hero);
        }
        for item in start.inventory.keys() {
            self.item("start", item);
        }
    }

    fn heroes(&mut self) {
        let Some(monster) = self.world.map.monsters.keys().next().cloned() else {
            self.report("no monsters".to_owned());
            return;
        };
        let Some(enemy) = self.world.monster_setup(&monster) else {
            return;
        };
        for (id, def) in &self.world.map.heroes {
            let owner = format!("hero `{id}`");
            if !self.world.map.classes.contains_key(&def.class) {
                self.report(format!("{owner}: unknown class `{}`", def.class));
                continue;
            }
            for piece in def.weapon.iter().chain(&def.armor) {
                self.piece(&owner, piece);
            }
            let learns = self
                .world
                .map
                .classes
                .get(&def.class)
                .map_or_else(Vec::new, |class| class.learns.clone());
            for learn in learns {
                let cost = self
                    .world
                    .actions
                    .get(&learn.action)
                    .map_or(0, |a| a.mp_cost);
                let state = HeroState::new(self.world, id, learn.level);
                let mp = self
                    .world
                    .hero_stats(&state)
                    .map_or(0, |stats| stats.max_mp);
                if cost > mp {
                    self.report(format!(
                        "{owner}: learns `{}` at level {} for {cost} MP, but has {mp} there",
                        learn.action, learn.level
                    ));
                }
            }
            for level in [1, MAX_LEVEL] {
                let state = HeroState::new(self.world, id, level);
                if let Some(setup) = self.world.hero_setup(&state) {
                    self.can_fight(
                        &format!("{owner} at level {level}"),
                        vec![setup],
                        enemy.clone(),
                    );
                }
            }
        }
    }

    fn monsters(&mut self) {
        let Some(hero) = self.world.map.start.party.first().cloned() else {
            return;
        };
        let state = HeroState::new(self.world, &hero, 1);
        let Some(party) = self.world.hero_setup(&state) else {
            return;
        };
        for (id, def) in &self.world.map.monsters {
            let owner = format!("monster `{id}`");
            if def.battle.name != format!("monster-{id}") {
                self.report(format!("{owner}: its name must be `monster-{id}`"));
            }
            for drop in &def.drops {
                self.item(&owner, &drop.item);
            }
            // The AI skips what it cannot pay for: such an action never shows.
            for action in &def.battle.actions {
                if let Some(cost) = self.world.actions.get(action).map(|a| a.mp_cost)
                    && cost > def.battle.stats.max_mp
                {
                    self.report(format!(
                        "{owner}: `{action}` costs {cost} MP, but it has {}",
                        def.battle.stats.max_mp
                    ));
                }
            }
            self.can_fight(&owner, vec![party.clone()], def.battle.clone());
        }
    }

    fn instances(&mut self) {
        for (id, instance) in &self.world.map.instances {
            let owner = format!("instance `{id}`");
            if !self.world.map.regions.contains_key(&instance.region) {
                self.report(format!("{owner}: unknown region `{}`", instance.region));
            }
            for trail in &instance.trails {
                for end in [&trail.a, &trail.b] {
                    if !instance.nodes.contains_key(end) {
                        self.report(format!("{owner}: a trail ends at unknown node `{end}`"));
                    }
                }
                if trail.chance > 100 {
                    self.report(format!("{owner}: trail chance above 100%"));
                }
                for group in &trail.encounters {
                    self.fight(&owner, &group.monsters);
                }
            }
            for (node_id, node) in &instance.nodes {
                let owner = format!("{owner}, node `{node_id}`");
                if node.x > 1000 || node.y > 1000 {
                    self.report(format!("{owner}: outside the 1000×1000 box"));
                }
                if let Some(event) = &node.event {
                    self.dialogue(&owner, &event.dialogue);
                }
                match &node.kind {
                    NodeKind::Path | NodeKind::Temple | NodeKind::Inn { .. } | NodeKind::Sealed => {
                    }
                    NodeKind::Board { quests } => {
                        for quest in quests {
                            self.quest(&owner, quest);
                        }
                    }
                    NodeKind::Exit { to } => self.position(&owner, to),
                    NodeKind::Battle {
                        monsters, on_win, ..
                    } => {
                        self.fight(&owner, monsters);
                        self.effects(&owner, on_win);
                    }
                    NodeKind::Chest {
                        items, equipment, ..
                    } => {
                        for item in items.keys() {
                            self.item(&owner, item);
                        }
                        for piece in equipment {
                            self.piece(&owner, piece);
                        }
                    }
                    NodeKind::Npc { talks, .. } => {
                        for talk in talks {
                            self.dialogue(&owner, &talk.dialogue);
                        }
                    }
                    NodeKind::Shop { shop } => {
                        if !self.world.map.shops.contains_key(shop) {
                            self.report(format!("{owner}: unknown shop `{shop}`"));
                        }
                    }
                }
            }
        }
    }

    fn dialogues(&mut self) {
        for (id, dialogue) in &self.world.map.dialogues {
            let owner = format!("dialogue `{id}`");
            if dialogue.lines.is_empty() {
                self.report(format!("{owner}: no lines"));
            }
            for choice in &dialogue.choices {
                self.effects(&owner, &choice.effects);
            }
            self.effects(&owner, &dialogue.effects);
            let choice_battles = dialogue.choices.iter().any(|choice| {
                choice
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, StoryEffect::Battle { .. }))
            });
            if choice_battles && !dialogue.effects.is_empty() {
                self.report(format!(
                    "{owner}: a choice that starts a battle leaves no room for the scene's effects"
                ));
            }
        }
    }

    fn shops(&mut self) {
        for (id, shop) in &self.world.map.shops {
            let owner = format!("shop `{id}`");
            for entry in &shop.items {
                self.item(&owner, &entry.id);
            }
            for entry in &shop.equipment {
                self.piece(&owner, &entry.id);
            }
        }
    }

    fn equipment(&mut self) {
        for (id, piece) in &self.world.map.equipment {
            for class in &piece.classes {
                if !self.world.map.classes.contains_key(class) {
                    self.report(format!("equipment `{id}`: unknown class `{class}`"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use engine::{ActionCategory, ActionDef, ActionId, Effect, TargetRule};

    use super::*;
    use crate::test_world::world;

    #[test]
    fn an_action_too_expensive_for_its_user_is_a_problem() {
        let mut world = (*world()).clone();
        let blast = ActionId::new("blast");
        world.actions.insert(
            blast.clone(),
            ActionDef {
                category: ActionCategory::Magic,
                target: TargetRule::SingleEnemy,
                mp_cost: 50,
                duration_ticks: 8,
                effect: Effect::MagicalDamage {
                    power: 8,
                    element: None,
                },
            },
        );
        for monster in world.map.monsters.values_mut() {
            monster.battle.actions.push(blast.clone());
        }
        for class in world.map.classes.values_mut() {
            class.learns.push(crate::def::Learn {
                level: 2,
                action: blast.clone(),
            });
        }

        let found = problems(&world);

        assert!(
            found
                .iter()
                .any(|p| p.contains("`blast` costs 50 MP, but it has 0"))
        );
        assert!(
            found
                .iter()
                .any(|p| p.contains("learns `blast` at level 2 for 50 MP"))
        );
        let clean = problems(&crate::test_world::world());
        assert!(!clean.iter().any(|p| p.contains(" MP")), "{clean:?}");
    }
}
