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

use std::fs;
use std::path::Path;

use crate::checks::support::{more_help, project_hint};
use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "port-availability";
pub(crate) const TITLE: &str = "Benötigte Ports verfügbar";

fn remediation() -> String {
    format!(
        concat!(
            "Den Prozess beenden, der die betroffenen Ports belegt, oder die Ports in ",
            "infra/docker/.env ändern.\n",
            "Den Prozess zu einem Port ermitteln (Beispiel: 8080):\n",
            "  macOS:   lsof -nP -iTCP:8080 -sTCP:LISTEN\n",
            "  Linux:   ss -tlnp 'sport = :8080'   (oder: lsof -i :8080)\n",
            "  Windows: netstat -ano | findstr :8080\n",
            "           anschließend: tasklist /FI \"PID eq <pid>\"\n",
            "Falls noch Container eines vorherigen Testhub-Laufs aktiv sind, ",
            "diese zuerst stoppen:\n",
            "  docker compose -f infra/docker/compose-local.yaml --profile full down\n",
            "Ein Port lässt sich ohne Änderung an .env über den Compose-Aufruf ändern:\n",
            "  PORT_CANOPY=18080 docker compose -f infra/docker/compose-local.yaml ",
            "--profile full up -d{0}"
        ),
        more_help()
    )
}

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let env_file = root.join("infra/docker/.env");
    if !env_file.exists() {
        return warn(
            ID,
            category,
            TITLE,
            "infra/docker/.env nicht gefunden – Portprüfung wird übersprungen.",
            Some(project_hint()),
        );
    }
    let content = match fs::read_to_string(&env_file) {
        Ok(content) => content,
        Err(error) => {
            return warn(
                ID,
                category,
                TITLE,
                format!("infra/docker/.env kann nicht gelesen werden: {error}"),
                None,
            );
        }
    };
    let ports: Vec<_> = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            if !key.trim().starts_with("PORT_") {
                return None;
            }
            Some((key.trim().to_string(), value.trim().parse::<u16>().ok()?))
        })
        .collect();
    if ports.is_empty() {
        return warn(
            ID,
            category,
            TITLE,
            "Keine PORT_*-Einträge in infra/docker/.env gefunden.",
            None,
        );
    }
    let used: Vec<_> = ports
        .iter()
        .filter(|(_, port)| std::net::TcpListener::bind(("127.0.0.1", *port)).is_err())
        .map(|(key, port)| format!("{key}={port}"))
        .collect();
    if !used.is_empty() {
        return error(
            ID,
            category,
            TITLE,
            format!(
                concat!("Folgende Ports sind bereits belegt:\n", "  {}"),
                used.join("\n  ")
            ),
            Some(remediation()),
        );
    }
    ok(
        ID,
        category,
        TITLE,
        format!(
            "Alle {} Ports aus infra/docker/.env sind verfügbar.",
            ports.len()
        ),
    )
}
