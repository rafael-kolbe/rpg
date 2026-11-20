//! `sim`: runs battles from a sandbox scenario with every combatant
//! AI-controlled.
//!
//! - One run (default) prints the battle log, tick by tick.
//! - `--runs N` plays N battles with seeds `seed..seed+N` and prints a
//!   summary: win rate, duration, HP left. This is the balancing tool.
//!
//! - `--campaign` plays the campaign's first arc with a bot instead (see
//!   `campaign.rs`); with `--runs N`, N games and a summary of each.
//!
//! ```text
//! cargo run -p sim -- --seed 42
//! cargo run -p sim -- --runs 1000
//! cargo run -p sim -- --campaign --runs 5
//! ```

mod campaign;

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use content::{Content, load_scenario};
use engine::{Battle, BattleEvent, CombatantId, Outcome, PartyControl, Side, TICK_MS};

#[derive(Debug, Parser)]
#[command(about = "Run AI-vs-AI battles from a sandbox scenario")]
struct Args {
    /// Sandbox scenario to play.
    #[arg(long, default_value = "sandbox/battle-basic.json")]
    scenario: PathBuf,
    /// Directory with actions.json and items.json.
    #[arg(long, default_value = "content")]
    content: PathBuf,
    /// Seed of the first battle.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Number of battles. 1 prints the full log; more print a summary.
    #[arg(long, default_value_t = 1)]
    runs: u64,
    /// Ticks after which a battle counts as a timeout (6000 = 10 minutes).
    #[arg(long, default_value_t = 6000)]
    max_ticks: u32,
    /// Play the campaign's first arc with a bot instead of a scenario.
    #[arg(long)]
    campaign: bool,
    /// Level of the heroes (not guests) in a campaign scenario.
    #[arg(long)]
    level: Option<u8>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let content = Content::load(&args.content)?;
    if args.campaign {
        let world = std::sync::Arc::new(content.load_world(&args.content)?);
        for seed in args.seed..args.seed + args.runs.max(1) {
            let report = campaign::play(std::sync::Arc::clone(&world), seed, args.runs <= 1)?;
            print!("seed {seed}: {}", report.describe());
        }
        return Ok(());
    }
    let world = content.load_world(&args.content)?;
    let encounter = load_scenario(&args.scenario)?
        .encounter(&world, args.level)
        .map_err(|problem| anyhow::anyhow!("{}: {problem}", args.scenario.display()))?;
    let new_battle = |seed| {
        Battle::new(
            content.battle_setup(encounter.clone()),
            seed,
            PartyControl::Ai,
        )
        .with_context(|| format!("invalid scenario {}", args.scenario.display()))
    };

    if args.runs <= 1 {
        let mut battle = new_battle(args.seed)?;
        play(&mut battle, args.max_ticks, true);
        return Ok(());
    }

    let mut report = Report::default();
    for seed in args.seed..args.seed + args.runs {
        let mut battle = new_battle(seed)?;
        play(&mut battle, args.max_ticks, false);
        report.add(&battle);
    }
    report.print(args.runs);
    Ok(())
}

/// Steps the battle to its end (or the tick limit), optionally printing events.
fn play(battle: &mut Battle, max_ticks: u32, log: bool) {
    while battle.outcome().is_none() && battle.tick() < max_ticks {
        let events = battle.step();
        if log {
            for event in &events {
                if let Some(line) = describe(battle, event) {
                    println!("[{:>6.1}s] {line}", seconds(battle.tick()));
                }
            }
        }
    }
    if log && battle.outcome().is_none() {
        println!("[{:>6.1}s] timeout", seconds(battle.tick()));
    }
}

fn seconds(ticks: u32) -> f64 {
    f64::from(ticks) * f64::from(TICK_MS) / 1000.0
}

fn name(battle: &Battle, id: CombatantId) -> String {
    battle
        .combatant(id)
        .map_or_else(|| id.to_string(), |c| format!("{} {id}", c.name()))
}

/// The log line of a thief's event.
fn describe_theft(battle: &Battle, event: &BattleEvent) -> Option<String> {
    let n = |id| name(battle, id);
    Some(match event {
        BattleEvent::Stole {
            actor,
            target,
            item,
        } => format!("  {} steals {item} from {}", n(*actor), n(*target)),
        BattleEvent::Fled { combatant } => format!("{} flees", n(*combatant)),
        BattleEvent::Recovered { combatant, item } => {
            format!("  {} drops the stolen {item}", n(*combatant))
        }
        _ => return None,
    })
}

/// One log line per event worth reading; gauge and queue bookkeeping is skipped.
fn describe(battle: &Battle, event: &BattleEvent) -> Option<String> {
    let n = |id| name(battle, id);
    Some(match event {
        BattleEvent::ActionStarted {
            actor,
            action,
            targets,
            mp_spent,
            ..
        } => {
            let targets: Vec<String> = targets.iter().map(|&t| n(t)).collect();
            let mp = if *mp_spent > 0 {
                format!(" ({mp_spent} MP)")
            } else {
                String::new()
            };
            format!("{} uses {action} on {}{mp}", n(*actor), targets.join(", "))
        }
        BattleEvent::Damaged {
            target,
            amount,
            critical,
            distracted,
            affinity,
        } => {
            let mut tags = Vec::new();
            if *critical {
                tags.push("critical".to_owned());
            }
            if *distracted {
                tags.push("distracted".to_owned());
            }
            if let Some(affinity) = affinity {
                tags.push(format!("{affinity:?}").to_lowercase());
            }
            let tags = if tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", tags.join(", "))
            };
            let hp = battle.combatant(*target).map_or(0, engine::Combatant::hp);
            format!("  {} takes {amount}{tags} -> {hp} HP", n(*target))
        }
        BattleEvent::Healed { target, amount } => format!("  {} recovers {amount} HP", n(*target)),
        BattleEvent::MpRestored { target, amount } => {
            format!("  {} recovers {amount} MP", n(*target))
        }
        BattleEvent::Revived { target, hp } => format!("  {} is revived with {hp} HP", n(*target)),
        BattleEvent::PoisonDamaged { target, amount } => {
            format!("{} loses {amount} HP to poison", n(*target))
        }
        BattleEvent::Regenerated { target, amount } => {
            format!("{} regenerates {amount} HP", n(*target))
        }
        BattleEvent::StatusApplied { target, status } => {
            format!("  {} gains {status:?}", n(*target))
        }
        BattleEvent::StatusImmune { target, status } => {
            format!("  {} is immune to {status:?}", n(*target))
        }
        BattleEvent::Missed { target } => format!("  miss on {}", n(*target)),
        BattleEvent::Covered {
            protector,
            protected,
        } => format!("  {} covers {}", n(*protector), n(*protected)),
        BattleEvent::Stole { .. } | BattleEvent::Fled { .. } | BattleEvent::Recovered { .. } => {
            describe_theft(battle, event)?
        }
        BattleEvent::StatusResisted { target, status } => {
            format!("  {} resists {status:?}", n(*target))
        }
        BattleEvent::StatusRemoved {
            target,
            status,
            reason,
        } => format!("{} loses {status:?} ({reason:?})", n(*target)),
        BattleEvent::Defending { combatant } => format!("{} defends", n(*combatant)),
        BattleEvent::ActionFailed {
            actor,
            action,
            reason,
            ..
        } => {
            format!("{} fails {action} ({reason:?})", n(*actor))
        }
        BattleEvent::ActionCancelled {
            actor,
            action,
            reason,
            ..
        } => {
            format!("{} cancels {action} ({reason:?})", n(*actor))
        }
        BattleEvent::KnockedOut { combatant } => format!("{} is knocked out", n(*combatant)),
        BattleEvent::BattleEnded { outcome } => format!("battle ended: {outcome:?}"),
        BattleEvent::TurnReady { .. } | BattleEvent::ActionQueued { .. } => return None,
    })
}

/// Aggregates many battles.
#[derive(Debug, Default)]
struct Report {
    victories: u64,
    defeats: u64,
    timeouts: u64,
    total_ticks: u64,
    party_hp_left_percent: u64,
}

impl Report {
    fn add(&mut self, battle: &Battle) {
        match battle.outcome() {
            Some(Outcome::Victory) => self.victories += 1,
            Some(Outcome::Defeat) => self.defeats += 1,
            None => self.timeouts += 1,
        }
        self.total_ticks += u64::from(battle.tick());

        let (hp, max) = battle
            .combatants()
            .iter()
            .filter(|c| c.side() == Side::Party)
            .fold((0u64, 0u64), |(hp, max), c| {
                (hp + u64::from(c.hp()), max + u64::from(c.stats().max_hp))
            });
        self.party_hp_left_percent += hp * 100 / max.max(1);
    }

    #[allow(clippy::cast_precision_loss)] // counts far below 2^52
    fn print(&self, runs: u64) {
        let percent = |count: u64| count as f64 * 100.0 / runs as f64;
        let average_seconds = self.total_ticks as f64 / runs as f64 * f64::from(TICK_MS) / 1000.0;
        println!("battles:        {runs}");
        println!(
            "victories:      {} ({:.1}%)",
            self.victories,
            percent(self.victories)
        );
        println!(
            "defeats:        {} ({:.1}%)",
            self.defeats,
            percent(self.defeats)
        );
        println!(
            "timeouts:       {} ({:.1}%)",
            self.timeouts,
            percent(self.timeouts)
        );
        println!("average length: {average_seconds:.1}s");
        println!(
            "party HP left:  {:.1}% on average",
            self.party_hp_left_percent as f64 / runs as f64
        );
    }
}
