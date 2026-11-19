//! A bot that plays the campaign's first arc from a new game to its last
//! scene, with the engine fighting every battle for the party. If the bot
//! can finish, a player can; its report (levels, defeats, battle time)
//! is the balancing tool for the arc.
//!
//! The bot follows a fixed route of objectives (`ROUTE`). It walks with
//! the world's own trails, rests at temples before bosses, buys potions and
//! better gear, and after a defeat grinds a few battles near the temple
//! before trying again.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;
use std::sync::Arc;

use anyhow::{Context, bail};
use engine::{Battle, ItemId, Outcome, PartyControl, Side};
use world::def::{Condition, NodeKind, Slot};
use world::view::SceneView;
use world::{
    BattleOrder, EquipmentId, Flag, Game, GameCommand, GameMode, NodeId, Position, Progress, Save,
    ShopGood, World,
};

/// Ticks after which a battle counts as lost (10 minutes).
const MAX_BATTLE_TICKS: u32 = 6000;
/// Tries per objective before the arc counts as unbeatable.
const MAX_TRIES: u32 = 12;
/// Wins to grind after a defeat.
const GRIND_WINS: u32 = 4;

/// What to do at a node.
#[derive(Debug, Clone, Copy)]
enum Goal {
    /// Walk there (its arrival scene plays).
    Visit,
    /// Walk there and interact (talk, open, fight, rest, take the exit).
    Use,
    /// Walk there and shop.
    Shop,
}

/// The first arc, in order: instance, node, goal.
const ROUTE: &[(&str, &str, Goal)] = &[
    ("brookhollow", "elder", Goal::Use),
    ("brookhollow", "store", Goal::Shop),
    ("brookhollow", "riverbank", Goal::Use),
    ("brookhollow", "old-mill", Goal::Use),
    ("brookhollow", "chapel", Goal::Use),
    ("brookhollow", "farm", Goal::Use),
    ("brookhollow", "chapel", Goal::Use),
    ("brookhollow", "north-road", Goal::Use),
    ("thornwood", "clearing", Goal::Visit),
    ("thornwood", "glade", Goal::Use),
    ("thornwood", "wayshrine", Goal::Use),
    ("thornwood", "trader", Goal::Shop),
    ("thornwood", "camp-gate", Goal::Use),
    ("thornwood", "camp-yard", Goal::Use),
    ("thornwood", "wayshrine", Goal::Use),
    ("thornwood", "trader", Goal::Shop),
    ("thornwood", "grukk-tent", Goal::Use),
    ("thornwood", "north-gate", Goal::Use),
    ("valcrest", "guild", Goal::Use),
    ("valcrest", "guard-post", Goal::Use),
    ("valcrest", "apothecary", Goal::Shop),
    ("valcrest", "armory", Goal::Shop),
    ("valcrest", "temple", Goal::Use),
    ("valcrest", "catacomb-gate", Goal::Use),
    ("catacombs-1", "side-crypt", Goal::Use),
    ("catacombs-1", "ossuary", Goal::Use),
    ("catacombs-1", "tomb-row", Goal::Use),
    ("catacombs-1", "stairs-down", Goal::Use),
    ("catacombs-2", "flooded-hall", Goal::Use),
    ("catacombs-2", "guard-room", Goal::Use),
    ("catacombs-2", "cells", Goal::Visit),
    ("catacombs-2", "stairs-down", Goal::Use),
    ("catacombs-3", "altar", Goal::Use),
    ("catacombs-3", "crypt-east", Goal::Use),
    ("catacombs-3", "crypt-west", Goal::Use),
    ("catacombs-3", "altar", Goal::Use),
    ("catacombs-3", "knight-hall", Goal::Use),
    ("catacombs-3", "altar", Goal::Use),
    ("catacombs-3", "boss-chamber", Goal::Use),
    ("catacombs-3", "hidden-stair", Goal::Use),
    ("valcrest", "guard-post", Goal::Use),
];

/// The flag the arc's last scene sets.
const ARC_COMPLETE: &str = "arc-complete";

/// How a run went.
#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    /// Battles fought.
    pub(crate) battles: u32,
    /// Battles lost.
    pub(crate) defeats: u32,
    /// Battle time, in ticks.
    pub(crate) battle_ticks: u64,
    /// Party levels when each instance was first entered.
    pub(crate) levels: Vec<(String, Vec<u8>)>,
    /// Defeats per objective (`instance/node`), when any.
    pub(crate) hard_spots: BTreeMap<String, u32>,
}

impl Report {
    /// Readable summary.
    pub(crate) fn describe(&self) -> String {
        let mut out = String::new();
        let minutes = self.battle_ticks * u64::from(engine::TICK_MS) / 60_000;
        let _ = writeln!(
            out,
            "{} battles, {} defeats, {minutes} min of battle",
            self.battles, self.defeats
        );
        for (instance, levels) in &self.levels {
            let _ = writeln!(out, "  entering {instance:<12} levels {levels:?}");
        }
        for (spot, defeats) in &self.hard_spots {
            let _ = writeln!(out, "  defeats at {spot}: {defeats}");
        }
        out
    }
}

/// One move on the way somewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    /// Walk a trail to this node.
    Move(NodeId),
    /// Take the exit the bot stands on.
    Exit,
}

/// Why walking stopped early.
enum Stop {
    /// The party lost a battle and is back at the temple.
    Defeated,
}

struct Bot {
    world: Arc<World>,
    game: Game,
    report: Report,
    log: bool,
}

/// Plays the arc with `seed`. Fails when the bot gets stuck or keeps losing.
pub(crate) fn play(world: Arc<World>, seed: u64, log: bool) -> anyhow::Result<Report> {
    let save = Save::new(&world, seed);
    let game = Game::new(
        Arc::clone(&world),
        GameMode::Campaign,
        Progress::default(),
        save,
    );
    let mut bot = Bot {
        world,
        game,
        report: Report::default(),
        log,
    };
    bot.settle()?;
    let mut entered = String::new();
    for &(instance, node, goal) in ROUTE {
        if instance != entered {
            instance.clone_into(&mut entered);
            let levels = bot.game.save().party.iter().map(|h| h.level).collect();
            bot.report.levels.push((entered.clone(), levels));
        }
        bot.objective(instance, node, goal)
            .with_context(|| format!("objective {instance}/{node}"))?;
    }
    if !bot.game.progress().has(&Flag::new(ARC_COMPLETE)) {
        bail!("the route ended without `{ARC_COMPLETE}`");
    }
    Ok(bot.report)
}

impl Bot {
    fn say(&self, line: &str) {
        if self.log {
            println!("{line}");
        }
    }

    fn here(&self) -> Position {
        self.game.save().position
    }

    fn objective(&mut self, instance: &str, node: &str, goal: Goal) -> anyhow::Result<()> {
        if self.log {
            let view = self.game.view();
            let party: Vec<_> = view
                .party
                .iter()
                .map(|h| {
                    format!(
                        "{} L{} {}/{} atk {} def {}",
                        h.hero, h.level, h.hp, h.max_hp, h.stats.attack, h.stats.defense
                    )
                })
                .collect();
            println!(
                "{instance}/{node}: {party:?}, {} crowns, {:?}",
                view.crowns, view.inventory
            );
        }
        let target = Position {
            instance: world::InstanceId::new(instance),
            node: NodeId::new(node),
        };
        for _ in 0..MAX_TRIES {
            match self.attempt(&target, goal)? {
                Ok(()) => {
                    // Chests and gifts may hold better gear.
                    return self.equip_best();
                }
                Err(Stop::Defeated) => {
                    *self
                        .report
                        .hard_spots
                        .entry(format!("{instance}/{node}"))
                        .or_insert(0) += 1;
                    self.grind()?;
                }
            }
        }
        bail!("lost {MAX_TRIES} times")
    }

    fn attempt(&mut self, target: &Position, goal: Goal) -> anyhow::Result<Result<(), Stop>> {
        if let Err(stop) = self.walk_to(target)? {
            return Ok(Err(stop));
        }
        match goal {
            Goal::Visit => Ok(Ok(())),
            Goal::Use => self.command(GameCommand::Interact { ambush: false }),
            Goal::Shop => {
                self.game.act(GameCommand::Interact { ambush: false })?;
                self.shop()?;
                self.game.act(GameCommand::Next)?;
                Ok(Ok(()))
            }
        }
    }

    /// Sends a command, fights what it starts, and clears the scenes after.
    fn command(&mut self, command: GameCommand) -> anyhow::Result<Result<(), Stop>> {
        let order = self
            .game
            .act(command.clone())
            .map_err(|error| anyhow::anyhow!("{command:?} refused: {error}"))?;
        drop(command);
        if let Some(order) = order
            && let Err(stop) = self.fight(order)?
        {
            return Ok(Err(stop));
        }
        self.settle_until_map()
    }

    /// Dialogues, rewards and notices until the map shows again; fights
    /// battles they start.
    fn settle_until_map(&mut self) -> anyhow::Result<Result<(), Stop>> {
        loop {
            let order = match self.game.view().scene {
                SceneView::Map => return Ok(Ok(())),
                SceneView::Dialogue { choices, .. } if choices > 0 => {
                    self.game.act(GameCommand::Choose { index: 0 })
                }
                SceneView::Defeat => {
                    self.game.act(GameCommand::Next)?;
                    return Ok(Err(Stop::Defeated));
                }
                SceneView::Shop { .. } | SceneView::Battle => bail!("unexpected scene"),
                SceneView::Dialogue { .. }
                | SceneView::Victory { .. }
                | SceneView::Rested
                | SceneView::Found { .. } => self.game.act(GameCommand::Next),
            }
            .map_err(|error| anyhow::anyhow!("scene refused: {error}"))?;
            if let Some(order) = order
                && let Err(stop) = self.fight(order)?
            {
                return Ok(Err(stop));
            }
        }
    }

    fn settle(&mut self) -> anyhow::Result<()> {
        match self.settle_until_map()? {
            Ok(()) => Ok(()),
            Err(Stop::Defeated) => bail!("defeated in an opening scene"),
        }
    }

    fn fight(&mut self, order: BattleOrder) -> anyhow::Result<Result<(), Stop>> {
        let enemies: Vec<_> = order
            .setup
            .encounter
            .enemies
            .iter()
            .map(|e| e.name.clone())
            .collect();
        let mut battle = Battle::new(order.setup, order.seed, PartyControl::Ai)
            .context("the game ordered an invalid battle")?;
        while battle.outcome().is_none() && battle.tick() < MAX_BATTLE_TICKS {
            battle.step();
        }
        let outcome = battle.outcome().unwrap_or(Outcome::Defeat);
        let party: Vec<_> = battle
            .combatants()
            .iter()
            .filter(|c| c.side() == Side::Party)
            .map(|c| (c.hp(), c.mp()))
            .collect();
        self.report.battles += 1;
        self.report.battle_ticks += u64::from(battle.tick());
        self.say(&format!(
            "  battle vs {enemies:?}: {outcome:?} in {:.0}s, party {party:?}",
            f64::from(battle.tick()) / 10.0
        ));
        self.game.finish_battle(outcome, &party, battle.inventory());
        if outcome == Outcome::Defeat {
            self.report.defeats += 1;
        }
        self.settle_until_map()
    }

    // --- walking -------------------------------------------------------------

    fn holds(&self, condition: &Condition) -> bool {
        let flags = &self.game.progress().flags;
        condition.requires.iter().all(|f| flags.contains(f))
            && !condition.forbids.iter().any(|f| flags.contains(f))
    }

    /// Where one step from `from` leads: trails of its instance, and its
    /// exit when it is one.
    fn neighbors(&self, from: &Position) -> Vec<(Position, Step)> {
        let Some(instance) = self.world.map.instances.get(&from.instance) else {
            return Vec::new();
        };
        let exists = |node: &NodeId| {
            instance
                .nodes
                .get(node)
                .is_some_and(|n| self.holds(&n.when))
        };
        let mut out = Vec::new();
        for trail in instance.trails.iter().filter(|t| self.holds(&t.when)) {
            let next = if trail.a == from.node {
                &trail.b
            } else if trail.b == from.node {
                &trail.a
            } else {
                continue;
            };
            if exists(next) {
                let position = Position {
                    instance: from.instance.clone(),
                    node: next.clone(),
                };
                out.push((position, Step::Move(next.clone())));
            }
        }
        if let Some(NodeKind::Exit { to }) = instance.nodes.get(&from.node).map(|n| &n.kind) {
            out.push((to.clone(), Step::Exit));
        }
        out
    }

    /// Shortest routes from here: for every reachable position, its distance
    /// and the first step toward it.
    fn routes(&self) -> BTreeMap<Position, (u32, Option<Step>)> {
        let here = self.here();
        let mut routes = BTreeMap::from([(here.clone(), (0, None))]);
        let mut queue = VecDeque::from([here]);
        while let Some(position) = queue.pop_front() {
            let (distance, first) = routes[&position].clone();
            for (next, step) in self.neighbors(&position) {
                if routes.contains_key(&next) {
                    continue;
                }
                routes.insert(
                    next.clone(),
                    (distance + 1, Some(first.clone().unwrap_or(step))),
                );
                queue.push_back(next);
            }
        }
        routes
    }

    fn walk_to(&mut self, target: &Position) -> anyhow::Result<Result<(), Stop>> {
        while &self.here() != target {
            let step = self
                .routes()
                .remove(target)
                .and_then(|(_, step)| step)
                .with_context(|| format!("no open path from {:?} to {target:?}", self.here()))?;
            self.patch_up()?;
            let command = match step {
                Step::Move(node) => GameCommand::Move { node },
                Step::Exit => GameCommand::Interact { ambush: false },
            };
            if let Err(stop) = self.command(command)? {
                return Ok(Err(stop));
            }
        }
        Ok(Ok(()))
    }

    /// Drinks potions outside battle for heroes below half HP.
    fn patch_up(&mut self) -> anyhow::Result<()> {
        let potion = ItemId::new("potion");
        let view = self.game.view();
        for (index, hero) in view.party.iter().enumerate() {
            if u32::from(hero.hp) * 2 < u32::from(hero.max_hp)
                && view.inventory.get(&potion).copied().unwrap_or(0) > 1
            {
                let hero = u8::try_from(index)?;
                self.game
                    .act(GameCommand::UseItem {
                        item: potion.clone(),
                        hero,
                    })
                    .ok();
            }
        }
        Ok(())
    }

    /// After a defeat (at the temple): wins a few battles on the nearest
    /// trail with monsters, then rests at the temple again.
    fn grind(&mut self) -> anyhow::Result<()> {
        let home = self.here();
        let routes = self.routes();
        let world = Arc::clone(&self.world);
        let (_, a, b) = world
            .map
            .instances
            .iter()
            .flat_map(|(id, instance)| {
                instance
                    .trails
                    .iter()
                    .filter(|t| t.chance > 0 && self.holds(&t.when))
                    .map(move |t| (id, t))
            })
            .filter_map(|(id, trail)| {
                let at = |node: &NodeId| Position {
                    instance: id.clone(),
                    node: node.clone(),
                };
                let (a, b) = (at(&trail.a), at(&trail.b));
                let distance = routes.get(&a)?.0.min(routes.get(&b)?.0);
                Some((distance, a, b))
            })
            .min_by_key(|(distance, ..)| *distance)
            .context("nowhere to grind")?;
        let start = self.report.battles - self.report.defeats;
        while self.report.battles - self.report.defeats < start + GRIND_WINS {
            for end in [&a, &b] {
                if let Err(Stop::Defeated) = self.walk_to(end)? {
                    self.say("  defeated while grinding");
                }
            }
            if self.report.defeats > 200 {
                bail!("grinding never ends");
            }
        }
        self.walk_to(&home)?.ok();
        let temple = world
            .map
            .instances
            .get(&home.instance)
            .and_then(|instance| instance.nodes.get(&home.node))
            .is_some_and(|node| node.kind == NodeKind::Temple);
        if temple {
            self.command(GameCommand::Interact { ambush: false })?.ok();
        }
        Ok(())
    }

    // --- shopping and gear ---------------------------------------------------

    fn shop(&mut self) -> anyhow::Result<()> {
        let SceneView::Shop { goods, .. } = self.game.view().scene else {
            bail!("not in a shop");
        };
        // Gear first: better weapon and armor for each hero, when affordable.
        for hero in self.game.view().party {
            for slot in [Slot::Weapon, Slot::Armor] {
                let worn = match slot {
                    Slot::Weapon => hero.weapon.clone(),
                    Slot::Armor => hero.armor.clone(),
                };
                let score = |id: &EquipmentId| self.score(id);
                let current = worn.as_ref().map_or(0, &score);
                let best = goods
                    .iter()
                    .filter_map(|entry| match &entry.good {
                        ShopGood::Equipment { id } => Some((id.clone(), entry.price)),
                        ShopGood::Item { .. } => None,
                    })
                    .filter(|(id, _)| {
                        self.world.map.equipment.get(id).is_some_and(|def| {
                            def.slot == slot && def.classes.iter().any(|c| c.as_str() == hero.class)
                        })
                    })
                    .filter(|(id, _)| score(id) > current)
                    .max_by_key(|(id, _)| score(id));
                if let Some((id, price)) = best
                    && price <= self.game.save().crowns
                {
                    self.game.act(GameCommand::Buy {
                        good: ShopGood::Equipment { id },
                        count: 1,
                    })?;
                }
            }
        }
        self.equip_best()?;
        // Then consumables.
        for (item, wanted) in [
            ("potion", 8_u16),
            ("hi-potion", 4),
            ("antidote", 3),
            ("ether", 2),
        ] {
            let good = ShopGood::Item {
                id: ItemId::new(item),
            };
            let Some(entry) = goods.iter().find(|entry| entry.good == good) else {
                continue;
            };
            let owned = self
                .game
                .save()
                .inventory
                .get(&ItemId::new(item))
                .copied()
                .unwrap_or(0);
            let affordable = self.game.save().crowns / entry.price.max(1);
            let count = wanted.saturating_sub(owned).min(u16::try_from(affordable)?);
            if count > 0 {
                self.game.act(GameCommand::Buy { good, count })?;
            }
        }
        Ok(())
    }

    fn score(&self, id: &EquipmentId) -> u16 {
        self.world.map.equipment.get(id).map_or(0, |def| {
            def.attack + def.defense + def.magic_defense + u16::from(def.evasion)
        })
    }

    /// Puts on the best piece in the bag for each hero and slot.
    fn equip_best(&mut self) -> anyhow::Result<()> {
        for (index, hero) in self.game.view().party.iter().enumerate() {
            for slot in [Slot::Weapon, Slot::Armor] {
                let worn = match slot {
                    Slot::Weapon => hero.weapon.clone(),
                    Slot::Armor => hero.armor.clone(),
                };
                let current = worn.as_ref().map_or(0, |id| self.score(id));
                let best = self
                    .game
                    .view()
                    .bag
                    .iter()
                    .filter(|entry| entry.slot == slot && entry.classes.contains(&hero.class))
                    .map(|entry| entry.id.clone())
                    .max_by_key(|id| self.score(id));
                if let Some(id) = best.filter(|id| self.score(id) > current) {
                    self.game.act(GameCommand::Equip {
                        hero: u8::try_from(index)?,
                        slot,
                        id: Some(id),
                    })?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn the_first_arc_can_be_finished_by_the_engine() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
        let content = content::Content::load(&dir).unwrap();
        let world = Arc::new(content.load_world(&dir).unwrap());

        let report = play(world, 42, false).unwrap_or_else(|error| panic!("{error:?}"));

        assert!(report.battles > 20, "{}", report.describe());
    }
}
