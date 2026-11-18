//! Sandbox scenarios (feature `sandbox`): loaded and validated once at
//! startup, so a broken scenario stops the server instead of failing mid-test.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, bail};
use content::{Content, load_encounter};
use engine::{Battle, Encounter, PartyControl};

/// Content definitions plus every scenario in the sandbox directory.
#[derive(Debug)]
pub(crate) struct Sandbox {
    pub(crate) content: Content,
    /// By name: the file name without `.json`.
    pub(crate) scenarios: BTreeMap<String, Encounter>,
}

impl Sandbox {
    pub(crate) fn load(content_dir: &Path, sandbox_dir: &Path) -> anyhow::Result<Self> {
        let content = Content::load(content_dir)?;
        let mut scenarios = BTreeMap::new();

        let entries = fs::read_dir(sandbox_dir)
            .with_context(|| format!("cannot read {}", sandbox_dir.display()))?;
        for entry in entries {
            let path = entry?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
                bail!("scenario file name is not UTF-8: {}", path.display());
            };
            let encounter = load_encounter(&path)?;
            Battle::new(
                content.battle_setup(encounter.clone()),
                0,
                PartyControl::Player,
            )
            .with_context(|| format!("invalid scenario {}", path.display()))?;
            scenarios.insert(name.to_owned(), encounter);
        }

        Ok(Self { content, scenarios })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_repository_scenarios() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let sandbox = Sandbox::load(&root.join("content"), &root.join("sandbox")).unwrap();

        assert!(sandbox.scenarios.contains_key("battle-basic"));
    }
}
