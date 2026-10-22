/*-
 * #%L
 * Testhub Doctor
 * %%
 * Copyright (C) 2026 gematik GmbH
 * %%
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * *******
 *
 * For additional notes and disclaimer from gematik and in case of changes
 * by gematik, find details in the "Readme" file.
 * #L%
 */

use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use testhub_doctor::{CheckCategory, CheckResult, Severity, check_ids, check_info, run_check};

#[derive(Debug, Parser)]
#[command(
    name = "testhub-doctor",
    version,
    about = "Prüft die Voraussetzungen zum Erstellen und Ausführen des TI-2.0-Testhub-Stacks.",
    flatten_help = true,
    after_help = "Ohne Unterbefehl werden alle Prüfungen im aktuellen Projektverzeichnis ausgeführt.\n\
                  Ausgabe- und Filteroptionen stehen nach 'check'.\n\n\
                  Beispiele:\n  \
                  testhub-doctor check --json\n  \
                  testhub-doctor --project-root /pfad/zum/testhub check --only compose-file,smcb-cert\n\n\
                  Hilfe zu einem Unterbefehl: testhub-doctor <BEFEHL> --help",
    arg_required_else_help = false
)]
struct Cli {
    #[arg(
        short = 'p',
        long,
        global = true,
        value_name = "PATH",
        default_value = ".",
        help = "Verzeichnis des Testhub-Projekts (Standard: aktuelles Verzeichnis)"
    )]
    project_root: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Alle Voraussetzungen prüfen und Ergebnisse anzeigen")]
    Check {
        #[arg(long, help = "Ergebnisse als JSON ausgeben (z. B. für CI-Pipelines)")]
        json: bool,
        #[arg(
            short = 'r',
            long,
            help = "Problembehebungen auch für Warnungen anzeigen (bei Fehlern immer)"
        )]
        show_remediation: bool,
        #[arg(long, help = "ANSI-Farbausgabe deaktivieren")]
        no_color: bool,
        #[arg(long, help = "ASCII-Zeichen statt Unicode-Symbole verwenden")]
        ascii: bool,
        #[arg(
            long,
            value_name = "ID,...",
            value_delimiter = ',',
            action = ArgAction::Append,
            help = "Kommagetrennte IDs der auszuführenden Prüfungen"
        )]
        only: Vec<String>,
    },
    #[command(name = "list-checks", about = "Alle verfügbaren Prüfungs-IDs anzeigen")]
    ListChecks,
    #[command(
        name = "generate-completion",
        about = "Shell-Vervollständigungsskripte erzeugen"
    )]
    GenerateCompletion { shell: Shell },
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Some(Commands::ListChecks) => {
            for (id, title) in check_ids() {
                println!("  {id:<20} {title}");
            }
            0
        }
        Some(Commands::GenerateCompletion { shell }) => {
            let mut command = Cli::command();
            generate(shell, &mut command, "testhub-doctor", &mut io::stdout());
            0
        }
        Some(Commands::Check {
            json,
            show_remediation,
            no_color,
            ascii,
            only,
        }) => run_cli(
            cli.project_root,
            json,
            show_remediation,
            no_color,
            ascii,
            only,
        ),
        None => run_cli(cli.project_root, false, false, false, false, Vec::new()),
    };
    std::process::exit(exit_code);
}

fn run_cli(
    project_root: PathBuf,
    json: bool,
    show_remediation: bool,
    no_color: bool,
    ascii: bool,
    only: Vec<String>,
) -> i32 {
    let project_root = match project_root.canonicalize() {
        Ok(path) if path.is_dir() => path,
        _ => {
            eprintln!(
                "FEHLER: Projektverzeichnis nicht gefunden: {}",
                project_root.display()
            );
            return 2;
        }
    };

    if !only.is_empty() {
        let known: Vec<_> = check_ids().into_iter().map(|(id, _)| id).collect();
        let unknown: Vec<_> = only
            .iter()
            .filter(|id| !known.contains(&id.as_str()))
            .collect();
        if !unknown.is_empty() {
            eprintln!(
                "FEHLER: Unbekannte Prüfungs-ID(s): {}",
                unknown
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            eprintln!("Verfügbare IDs: {}", known.join(", "));
            return 2;
        }
    }

    let selected: Vec<_> = check_ids()
        .into_iter()
        .filter(|(id, _)| only.is_empty() || only.iter().any(|selected| selected == id))
        .map(|(id, _)| id)
        .collect();
    let results = if json {
        selected
            .iter()
            .filter_map(|id| run_check(id, &project_root))
            .collect()
    } else {
        render_terminal_start(no_color);
        let mut results = Vec::with_capacity(selected.len());
        let mut current_category = None;
        for id in &selected {
            if let Some((_, title, category)) = check_info(id) {
                if current_category != Some(category) {
                    render_category(category);
                    current_category = Some(category);
                }
                render_running(title, no_color, ascii);
            }
            if let Some(result) = run_check(id, &project_root) {
                render_result(&result, show_remediation, no_color, ascii);
                results.push(result);
            }
        }
        results
    };

    if json {
        println!("{}", render_json(&results));
    } else {
        render_summary(&results, show_remediation, no_color, ascii);
    }
    if results
        .iter()
        .any(|result| result.severity == Severity::Error)
    {
        1
    } else {
        0
    }
}

fn render_json(results: &[CheckResult]) -> String {
    let passed = results
        .iter()
        .filter(|result| result.severity == Severity::Ok)
        .count();
    let warnings = results
        .iter()
        .filter(|result| result.severity == Severity::Warn)
        .count();
    let errors = results
        .iter()
        .filter(|result| result.severity == Severity::Error)
        .count();
    let skipped = results
        .iter()
        .filter(|result| result.severity == Severity::Skipped)
        .count();
    let status = if errors > 0 {
        "ERROR"
    } else if warnings > 0 {
        "WARN"
    } else {
        "OK"
    };
    serde_json::to_string_pretty(&serde_json::json!({
        "summary": {
            "passed": passed,
            "warnings": warnings,
            "errors": errors,
            "skipped": skipped,
            "status": status
        },
        "checks": results,
    }))
    .expect("serializing check results cannot fail")
}

fn render_terminal_start(no_color: bool) {
    let color = !no_color && io::stdout().is_terminal();
    println!(
        "\n{}\n{}\n{}\n",
        style(" +------------------------------------------+", "1", color),
        style(" | Testhub Doctor - Voraussetzungen prüfen  |", "1", color),
        style(" +------------------------------------------+", "1", color)
    );
}

fn render_category(category: CheckCategory) {
    let name = category.display_name();
    println!(
        "\n  -- {name} {}",
        "-".repeat(40usize.saturating_sub(name.len()).max(2))
    );
}

fn render_running(title: &str, no_color: bool, force_ascii: bool) {
    if io::stdout().is_terminal() {
        let marker = if force_ascii { "[....]" } else { "…" };
        print!("  {}  {title} ...\r", style(marker, "90", !no_color));
        use std::io::Write;
        let _ = io::stdout().flush();
    }
}

fn render_result(result: &CheckResult, show_remediation: bool, no_color: bool, force_ascii: bool) {
    let color = !no_color && io::stdout().is_terminal();
    let ascii = force_ascii || !io::stdout().is_terminal();
    let (icon, code) = match result.severity {
        Severity::Ok => (if ascii { "[OK]  " } else { "✓" }, "32"),
        Severity::Warn => (if ascii { "[WARN]" } else { "⚠" }, "33"),
        Severity::Error => (if ascii { "[FAIL]" } else { "✗" }, "31"),
        Severity::Skipped => (if ascii { "[SKIP]" } else { "↷" }, "90"),
    };
    println!(
        "\r  {}  {}            ",
        style(icon, code, color),
        style(&result.title, code, color)
    );
    for line in result.detail.lines() {
        println!("       {}", style(line, "90", color));
    }
    if let Some(remedy) = &result.remediation
        && (result.severity == Severity::Error
            || (show_remediation && result.severity == Severity::Warn))
    {
        println!("\n     {}", style("Problembehebung:", "1", color));
        for line in remedy.lines() {
            println!("       {line}");
        }
        println!();
    }
}

fn render_summary(
    results: &[CheckResult],
    show_remediation: bool,
    no_color: bool,
    force_ascii: bool,
) {
    let color = !no_color && io::stdout().is_terminal();
    let ascii = force_ascii || !io::stdout().is_terminal();
    let (ok_icon, warn_icon, err_icon, skipped_icon) = if ascii {
        ("[OK]  ", "[WARN]", "[FAIL]", "[SKIP]")
    } else {
        ("✓", "⚠", "✗", "↷")
    };
    let passed = results
        .iter()
        .filter(|result| result.severity == Severity::Ok)
        .count();
    let warnings = results
        .iter()
        .filter(|result| result.severity == Severity::Warn)
        .count();
    let errors = results
        .iter()
        .filter(|result| result.severity == Severity::Error)
        .count();
    let skipped = results
        .iter()
        .filter(|result| result.severity == Severity::Skipped)
        .count();
    println!(
        "\n{}",
        style(
            &format!(" --- Zusammenfassung {}", "-".repeat(28)),
            "1",
            color
        )
    );
    println!(
        "  {}",
        style(&format!("{ok_icon}  Erfolgreich: {passed}"), "32", color)
    );
    if warnings > 0 {
        println!(
            "  {}",
            style(
                &format!("{warn_icon}  Warnungen  : {warnings}"),
                "33",
                color
            )
        );
    }
    if errors > 0 {
        println!(
            "  {}",
            style(&format!("{err_icon}  Fehler     : {errors}"), "31", color)
        );
    }
    if skipped > 0 {
        println!(
            "  {}",
            style(
                &format!("{skipped_icon}  Übersprungen: {skipped}"),
                "90",
                color
            )
        );
    }
    println!();
    if errors == 0 && warnings == 0 {
        println!(
            concat!(
                "{}\n\n  Nächste Schritte:\n",
                "    ./mvnw clean install -Pdocker -DskipTests\n",
                "    docker compose -f infra/docker/compose-local.yaml ",
                "--profile full up -d --remove-orphans\n"
            ),
            style(
                "  Alle anwendbaren Prüfungen erfolgreich! Der Testhub kann gestartet werden.",
                "1;32",
                color
            )
        );
    } else if errors == 0 {
        println!(
            "{}",
            style(
                concat!(
                    "  Es liegen Warnungen vor. Der Testhub kann möglicherweise trotzdem ",
                    "gestartet werden; bitte die Hinweise prüfen."
                ),
                "1;33",
                color
            )
        );
        if !show_remediation {
            println!(concat!(
                "  Mit --show-remediation (-r) werden auch Problembehebungen für ",
                "Warnungen angezeigt."
            ));
        }
    } else {
        println!(
            "{}\n  Problembehebungen zu fehlgeschlagenen Prüfungen stehen jeweils oberhalb.\n",
            style(
                &format!("  Vor dem Start des Testhubs müssen {errors} Fehler behoben werden."),
                "1;31",
                color
            )
        );
    }
}

fn style(text: &str, code: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, render_json};
    use clap::{CommandFactory, Parser, error::ErrorKind};
    use std::path::PathBuf;
    use testhub_doctor::{CheckCategory, CheckResult, Severity};

    #[test]
    fn main_help_includes_check_options_and_example() {
        let help = Cli::command().render_help().to_string();
        for option in [
            "--json",
            "--show-remediation",
            "--no-color",
            "--ascii",
            "--only",
        ] {
            assert!(help.contains(option), "missing option: {option}");
        }
        assert!(help.contains("testhub-doctor check --json"));
        assert!(help.contains("testhub-doctor <BEFEHL> --help"));
    }

    #[test]
    fn check_accepts_documented_options_and_global_project_root_positions() {
        for arguments in [
            vec!["testhub-doctor", "--project-root", "testhub", "check"],
            vec!["testhub-doctor", "check", "-p", "testhub"],
        ] {
            let cli = Cli::try_parse_from(arguments.into_iter().chain([
                "--json",
                "--only",
                "java-version,java-home-path",
                "--only",
                "docker-daemon",
                "-r",
                "--no-color",
                "--ascii",
            ]))
            .unwrap();
            assert_eq!(cli.project_root, PathBuf::from("testhub"));
            let Some(Commands::Check {
                json,
                show_remediation,
                no_color,
                ascii,
                only,
            }) = cli.command
            else {
                panic!("expected check subcommand");
            };
            assert!(json && show_remediation && no_color && ascii);
            assert_eq!(only, ["java-version", "java-home-path", "docker-daemon"]);
        }
        let cli = Cli::try_parse_from(["testhub-doctor", "-p", "testhub"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.project_root, PathBuf::from("testhub"));
        assert_eq!(
            Cli::try_parse_from(["testhub-doctor", "--json"])
                .unwrap_err()
                .kind(),
            ErrorKind::UnknownArgument
        );
    }

    #[test]
    fn json_report_contains_documented_fields_counts_and_status_precedence() {
        let mut results = Vec::new();
        for (severity, status) in [
            (Severity::Skipped, "OK"),
            (Severity::Ok, "OK"),
            (Severity::Warn, "WARN"),
            (Severity::Error, "ERROR"),
        ] {
            results.push(CheckResult {
                check_id: "example".into(),
                category: CheckCategory::System,
                title: "Example".into(),
                severity,
                detail: "Detail".into(),
                remediation: (severity == Severity::Warn).then(|| "Remedy".into()),
            });
            let report: serde_json::Value = serde_json::from_str(&render_json(&results)).unwrap();
            assert_eq!(report["summary"]["status"], status);
        }
        let report: serde_json::Value = serde_json::from_str(&render_json(&results)).unwrap();
        assert_eq!(
            report["summary"],
            serde_json::json!({
                "passed": 1, "warnings": 1, "errors": 1, "skipped": 1, "status": "ERROR"
            })
        );
        assert_eq!(
            report["checks"][0],
            serde_json::json!({
                "id": "example", "category": "SYSTEM", "title": "Example",
                "severity": "SKIPPED", "detail": "Detail", "remediation": null
            })
        );
        assert_eq!(report["checks"][2]["remediation"], "Remedy");
    }
}
