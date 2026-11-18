//! Repository-wide content validation, run by `make check`.
//!
//! Catches what the type system cannot: ids that break the naming rule,
//! scenarios the engine would reject, and names missing from a locale.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use engine::{Battle, PartyControl};
use fluent_syntax::ast::Entry;

use utils::text::is_kebab_case;

use crate::{Content, load_encounter};

/// What a successful check covered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    /// Actions checked.
    pub actions: usize,
    /// Items checked.
    pub items: usize,
    /// Sandbox scenarios checked.
    pub scenarios: usize,
    /// Locales checked.
    pub locales: usize,
}

/// Checks the content, the sandbox scenarios and their translations under the
/// repository `root`.
///
/// # Errors
///
/// Every problem found, one line each.
pub fn check(root: &Path) -> Result<Summary, Vec<String>> {
    let mut problems = Vec::new();

    let content = match Content::load(&root.join("content")) {
        Ok(content) => content,
        Err(error) => return Err(vec![error.to_string()]),
    };
    let locales = match load_locales(&root.join("locales")) {
        Ok(locales) => locales,
        Err(error) => return Err(vec![error]),
    };

    let mut needed: BTreeSet<String> = BTreeSet::new();
    for id in content.actions.keys() {
        check_id("action", id.as_str(), &mut problems);
        needed.insert(format!("action-{id}"));
    }
    for id in content.items.keys() {
        check_id("item", id.as_str(), &mut problems);
        needed.insert(format!("item-{id}"));
    }

    let scenarios = scenario_files(&root.join("sandbox"), &mut problems);
    for path in &scenarios {
        let name = path.display();
        let encounter = match load_encounter(path) {
            Ok(encounter) => encounter,
            Err(error) => {
                problems.push(error.to_string());
                continue;
            }
        };
        for member in encounter.party.iter().chain(&encounter.enemies) {
            needed.insert(member.name.clone());
        }
        let party_has_ai = encounter.party.iter().all(|member| member.ai.is_some());
        let controls: &[PartyControl] = if party_has_ai {
            &[PartyControl::Player, PartyControl::Ai]
        } else {
            &[PartyControl::Player]
        };
        for &control in controls {
            if let Err(error) = Battle::new(content.battle_setup(encounter.clone()), 0, control) {
                problems.push(format!("{name}: {error}"));
            }
        }
    }

    for (locale, ids) in &locales {
        for id in needed.difference(ids) {
            problems.push(format!("locales/{locale}: missing message `{id}`"));
        }
    }

    if problems.is_empty() {
        Ok(Summary {
            actions: content.actions.len(),
            items: content.items.len(),
            scenarios: scenarios.len(),
            locales: locales.len(),
        })
    } else {
        Err(problems)
    }
}

/// Content ids are lowercase kebab-case: `iron-sword`, `forest-wolf`.
fn check_id(kind: &str, id: &str, problems: &mut Vec<String>) {
    if !is_kebab_case(id) {
        problems.push(format!("{kind} id `{id}` is not lowercase kebab-case"));
    }
}

fn scenario_files(dir: &Path, problems: &mut Vec<String>) -> Vec<std::path::PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            problems.push(format!("cannot read {}: {error}", dir.display()));
            return Vec::new();
        }
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

/// Message ids defined by each locale directory.
fn load_locales(dir: &Path) -> Result<Vec<(String, BTreeSet<String>)>, String> {
    let read = |path: &Path| {
        fs::read_dir(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
    };

    let mut locales = Vec::new();
    for entry in read(dir)?.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut ids = BTreeSet::new();
        for file in read(&path)?.filter_map(Result::ok).map(|e| e.path()) {
            if file.extension().is_none_or(|ext| ext != "ftl") {
                continue;
            }
            let source = fs::read_to_string(&file)
                .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
            let resource =
                fluent_syntax::parser::parse(source.as_str()).map_err(|(_, errors)| {
                    format!("{}: {} syntax error(s)", file.display(), errors.len())
                })?;
            for entry in resource.body {
                if let Entry::Message(message) = entry {
                    ids.insert(message.id.name.to_owned());
                }
            }
        }
        let locale = entry.file_name().to_string_lossy().into_owned();
        locales.push((locale, ids));
    }
    locales.sort();
    Ok(locales)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_kebab_case_ids_only() {
        let mut problems = Vec::new();

        for id in ["potion", "revive-feather", "fire-2"] {
            check_id("item", id, &mut problems);
        }
        assert_eq!(problems, Vec::<String>::new());

        for id in ["Potion", "revive_feather", "-fire", "a--b", ""] {
            check_id("item", id, &mut problems);
        }
        assert_eq!(problems.len(), 5);
    }
}
