//! `content-check [ROOT]`: validates content, sandbox scenarios and their
//! translations under the repository root (default: current directory).
//! Exits with 1 and one line per problem when anything is wrong.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("."), PathBuf::from);

    match content::check(&root) {
        Ok(summary) => {
            println!(
                "Content ok: {} actions, {} items, {} scenarios, {} instances, {} monsters, {} locales.",
                summary.actions,
                summary.items,
                summary.scenarios,
                summary.instances,
                summary.monsters,
                summary.locales
            );
            ExitCode::SUCCESS
        }
        Err(problems) => {
            eprintln!("Content check failed ({}):", problems.len());
            for problem in problems {
                eprintln!("  - {problem}");
            }
            ExitCode::FAILURE
        }
    }
}
