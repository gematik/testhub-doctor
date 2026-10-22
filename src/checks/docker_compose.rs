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
use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "docker-compose";
pub(crate) const TITLE: &str = "Docker-Compose-v2-Plugin verfügbar";

fn install_compose() -> String {
    concat!(
        "Das Docker-Compose-v2-Plugin installieren ('docker compose', nicht ",
        "'docker-compose'):\n",
        "  macOS/Windows: Docker Desktop aktualisieren – Compose v2 ist enthalten.\n",
        "  Linux:         sudo apt install docker-compose-plugin\n",
        "                 – oder der Anleitung unter ",
        "https://docs.docker.com/compose/install/linux/ folgen\n",
        "Das ältere Python-basierte 'docker-compose' (v1) wird NICHT unterstützt."
    )
    .into()
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    let output = run_command(&["docker", "compose", "version"], Duration::from_secs(10));
    if !output.succeeded() {
        let legacy = run_command(&["docker-compose", "--version"], Duration::from_secs(10));
        let note = if legacy.succeeded() {
            format!(
                concat!(
                    "\nHinweis: Älteres 'docker-compose' gefunden ({}), ",
                    "Version 1 wird jedoch nicht unterstützt."
                ),
                legacy.stdout
            )
        } else {
            String::new()
        };
        return error(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "'docker compose' (v2-Plugin) ist nicht verfügbar.\n",
                    "{}{}"
                ),
                output.combined(),
                note
            ),
            Some(install_compose()),
        );
    }
    if let Some((major, minor)) = parse_compose_version(&output.stdout)
        && (major < 2 || (major == 2 && minor < 30))
    {
        return warn(
            ID,
            category,
            TITLE,
            format!("Compose {major}.{minor} gefunden; empfohlen wird Version 2.30 oder neuer."),
            Some(install_compose()),
        );
    }
    ok(
        ID,
        category,
        TITLE,
        output
            .stdout
            .replace("Docker Compose version", "Docker-Compose-Version"),
    )
}

fn parse_compose_version(text: &str) -> Option<(u32, u32)> {
    let marker = "Docker Compose version ";
    let rest = text
        .get(text.find(marker)? + marker.len()..)?
        .strip_prefix('v')
        .unwrap_or(&text[text.find(marker)? + marker.len()..]);
    let mut parts = rest.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::parse_compose_version;

    #[test]
    fn compose_version_parser_handles_optional_v() {
        assert_eq!(
            parse_compose_version("Docker Compose version v2.30.1"),
            Some((2, 30))
        );
        assert_eq!(
            parse_compose_version("Docker Compose version 2.29.7"),
            Some((2, 29))
        );
        assert_eq!(parse_compose_version("unknown"), None);
    }
}
