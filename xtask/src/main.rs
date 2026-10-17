// SPDX-License-Identifier: GPL-3.0-or-later
use std::process::{Command, ExitCode};

enum Task {
    VerifyContracts,
    GenerateContracts,
    Check,
}
impl Task {
    fn parse(value: Option<String>) -> Result<Self, String> {
        match value.as_deref() {
            Some("contracts") => Ok(Self::VerifyContracts),
            Some("generate-contracts") => Ok(Self::GenerateContracts),
            Some("check") => Ok(Self::Check),
            _ => Err("usage: cargo run -p xtask -- [contracts|generate-contracts|check]".into()),
        }
    }
    fn run(self) -> Result<(), String> {
        match self {
            Self::VerifyContracts => kit_build::verify_contracts().map_err(|e| e.to_string()),
            Self::GenerateContracts => std::fs::write(
                kit_build::ui_directory().join("shared-contracts.slint"),
                kit_build::shared_contract_source(),
            )
            .map_err(|e| e.to_string()),
            Self::Check => run("cargo", &["fmt", "--all", "--", "--check"])
                .and_then(|_| kit_build::verify_contracts().map_err(|e| e.to_string()))
                .and_then(|_| run("cargo", &["test", "--workspace", "--locked"]))
                .and_then(|_| {
                    run(
                        "cargo",
                        &[
                            "clippy",
                            "--workspace",
                            "--all-targets",
                            "--locked",
                            "--",
                            "-D",
                            "warnings",
                        ],
                    )
                }),
        }
    }
}
fn main() -> ExitCode {
    match Task::parse(std::env::args().nth(1)).and_then(Task::run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
fn run(command: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(command)
        .args(args)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command} {} failed: {status}", args.join(" ")))
    }
}
