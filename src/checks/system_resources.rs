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
 * #%L
 */

use fs2::available_space;
use std::env;
use std::path::{Path, PathBuf};
use sysinfo::System;

use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult, Severity};

pub(crate) const ID: &str = "system-resources";
pub(crate) const TITLE: &str = "Systemressourcen (Arbeitsspeicher und Speicherplatz)";

fn free_disk() -> String {
    concat!(
        "Speicherplatz freigeben – Docker-Images für den vollständigen Stack benötigen ",
        "mehrere GB.\n",
        "  docker system df           # Speicherverbrauch von Docker anzeigen\n",
        "  docker system prune -a     # ungenutzte Images, Container und Netzwerke entfernen\n",
        "  docker volume prune        # verwaiste Volumes entfernen (Achtung: Datenverlust ",
        "möglich)\n",
        "Unter macOS/Windows auch die Speicherplatzbegrenzung in Docker Desktop prüfen\n",
        "(Einstellungen → Ressourcen → Größe des Disk-Images)."
    )
    .into()
}

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let mut system = System::new();
    system.refresh_memory();
    let total_mb = system.total_memory() / (1024 * 1024);
    let ram_result = if total_mb == 0 {
        warn(
            ID,
            category,
            TITLE,
            "Der gesamte Arbeitsspeicher kann auf dieser Plattform nicht ermittelt werden.",
            None,
        )
    } else if total_mb < 6 * 1024 {
        error(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "Nur {} MB physischer Arbeitsspeicher erkannt. Für den ",
                    "vollständigen Stack mit 23 Containern werden mindestens 8192 MB ",
                    "empfohlen."
                ),
                total_mb
            ),
            Some(
                concat!(
                    "Mehr Arbeitsspeicher bereitstellen oder ein reduziertes Profil starten:\n",
                    "  docker compose -f infra/docker/compose-local.yaml --profile vsdm up -d"
                )
                .into(),
            ),
        )
    } else if total_mb < 8 * 1024 {
        warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "{} MB Arbeitsspeicher erkannt – für den vollständigen Stack werden ",
                    "8192 MB empfohlen."
                ),
                total_mb
            ),
            Some(
                concat!(
                    "Vor dem Start aller Container andere Anwendungen schließen oder ein\n",
                    "reduziertes Docker-Compose-Profil verwenden. Unter macOS/Windows ",
                    "zusätzlich\n",
                    "die Speicherbegrenzung in Docker Desktop prüfen ",
                    "(Settings → Resources → Memory)."
                )
                .into(),
            ),
        )
    } else {
        ok(
            ID,
            category,
            TITLE,
            format!("{total_mb} MB Arbeitsspeicher – ausreichend."),
        )
    };

    let project_free = free_gb(root);
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root.to_path_buf());
    let home_free = free_gb(&home);
    let min_free = project_free.min(home_free);
    let disk_result = if min_free < 0 {
        warn(
            ID,
            category,
            TITLE,
            "Der verfügbare Speicherplatz kann nicht ermittelt werden.",
            None,
        )
    } else if min_free < 10 {
        warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "Wenig freier Speicherplatz: {} GB verfügbar (Projekt: {} GB, ",
                    "Benutzerverzeichnis: {} GB). Empfohlen werden 10 GB."
                ),
                min_free, project_free, home_free
            ),
            Some(free_disk()),
        )
    } else {
        ok(
            ID,
            category,
            TITLE,
            format!("Speicherplatz ausreichend ({min_free} GB frei)."),
        )
    };
    if severity_order(disk_result.severity) > severity_order(ram_result.severity) {
        disk_result
    } else {
        ram_result
    }
}

fn free_gb(path: &Path) -> i64 {
    available_space(path)
        .map(|bytes| (bytes / (1024 * 1024 * 1024)) as i64)
        .unwrap_or(-1)
}

fn severity_order(severity: Severity) -> u8 {
    match severity {
        Severity::Ok | Severity::Skipped => 0,
        Severity::Warn => 1,
        Severity::Error => 2,
    }
}
