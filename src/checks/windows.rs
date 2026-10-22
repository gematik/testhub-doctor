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

use std::time::Duration;

use crate::process::run_command;
use crate::results::{error, ok, skipped, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "windows-wsl";
pub(crate) const TITLE: &str = "Windows-/WSL-2-Konfiguration";

fn remedy() -> String {
    concat!(
        "WSL 2 unter Windows aktivieren (erforderlich für Docker Desktop):\n",
        "  1. PowerShell als Administrator öffnen und ausführen:  wsl --install\n",
        "  2. Computer neu starten.\n",
        "  3. In Docker Desktop → Settings → General die Option\n",
        "     \"Use the WSL 2 based engine\" aktivieren.\n",
        "Weitere Informationen: https://learn.microsoft.com/windows/wsl/install\n",
        "Häufige Hindernisse:\n",
        "  - Virtualisierung muss im BIOS/UEFI aktiviert sein (Intel VT-x / AMD-V).\n",
        "  - Auf verwalteten Geräten kann eine Gruppenrichtlinie WSL blockieren – ",
        "IT kontaktieren."
    )
    .into()
}

fn set_wsl_two() -> String {
    concat!(
        "WSL 2 als Standard festlegen:\n",
        "  wsl --set-default-version 2\n",
        "Anschließend in Docker Desktop → Einstellungen → Allgemein die ",
        "WSL-2-basierte Engine aktivieren."
    )
    .into()
}

fn enable_wsl_backend() -> String {
    concat!(
        "In Docker Desktop → Einstellungen → Allgemein die Option ",
        "'WSL-2-basierte Engine verwenden' aktivieren.\n",
        "Das Hyper-V-Backend funktioniert ebenfalls, WSL 2 bietet jedoch eine ",
        "bessere Leistung."
    )
    .into()
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    if !cfg!(windows) {
        return skipped(ID, category, TITLE, "Nur unter Windows anwendbar.");
    }
    let output = run_command(&["wsl.exe", "--status"], Duration::from_secs(15));
    if !output.succeeded() {
        return error(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "WSL wurde nicht gefunden oder 'wsl --status' ist fehlgeschlagen:\n",
                    "{}"
                ),
                output.combined()
            ),
            Some(remedy()),
        );
    }
    let status = output.combined().to_lowercase();
    if !status.contains("wsl 2") && !status.contains("version: 2") {
        return warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "WSL ist installiert, aber WSL 2 scheint nicht als Standardversion ",
                    "eingerichtet zu sein:\n{}"
                ),
                output.stdout
            ),
            Some(set_wsl_two()),
        );
    }
    let docker = run_command(&["docker", "info"], Duration::from_secs(20));
    if docker.succeeded() && !docker.stdout.to_lowercase().contains("wsl") {
        return warn(
            ID,
            category,
            TITLE,
            concat!(
                "WSL 2 ist verfügbar, Docker verwendet möglicherweise jedoch nicht das ",
                "WSL-2-Backend."
            ),
            Some(enable_wsl_backend()),
        );
    }
    ok(
        ID,
        category,
        TITLE,
        concat!(
            "WSL 2 ist als Standardversion festgelegt und Docker scheint das ",
            "WSL-2-Backend zu verwenden."
        ),
    )
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::check;
    use crate::{CheckCategory, Severity};

    #[test]
    fn windows_check_is_marked_skipped_on_non_windows() {
        let result = check(CheckCategory::Windows);
        assert_eq!(result.severity, Severity::Skipped);
        assert_eq!(result.detail, "Nur unter Windows anwendbar.");
    }
}
