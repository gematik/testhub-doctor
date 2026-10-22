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

use super::support::more_help;
use crate::process::run_command;
use crate::results::{error, ok};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "docker-daemon";
pub(crate) const TITLE: &str = "Docker installiert und Daemon aktiv";

fn install_docker() -> String {
    format!(
        concat!(
            "Docker installieren:\n",
            "  macOS:   Docker Desktop – https://www.docker.com/products/docker-desktop/\n",
            "           oder Rancher Desktop (kostenlos und quelloffen) – ",
            "https://rancherdesktop.io/\n",
            "  Ubuntu:  sudo apt install docker.io docker-compose-plugin\n",
            "           sudo systemctl enable --now docker\n",
            "  Windows: Docker Desktop mit WSL-2-Backend\n",
            "           (winget install Docker.DockerDesktop){0}"
        ),
        more_help()
    )
}

fn docker_permission() -> String {
    format!(
        concat!(
            "Der Docker-Daemon ist aktiv, aber der Zugriff darauf ist nicht erlaubt.\n",
            "  Linux: Das Benutzerkonto zur Gruppe docker hinzufügen und erneut anmelden:\n",
            "           sudo usermod -aG docker $USER\n",
            "           newgrp docker    # alternativ ab- und wieder anmelden\n",
            "Den Testhub-Build nicht mit sudo ausführen, sondern die ",
            "Gruppenmitgliedschaft korrigieren.{0}"
        ),
        more_help()
    )
}

fn start_docker() -> String {
    format!(
        concat!(
            "Die Docker-Befehlszeile ist installiert, aber der Daemon ist nicht ",
            "erreichbar.\n",
            "  macOS:   Docker Desktop (oder Rancher Desktop) öffnen und warten, bis das\n",
            "           Symbol in der Menüleiste nicht mehr animiert.\n",
            "  Linux:   sudo systemctl start docker\n",
            "  Windows: Docker Desktop über das Startmenü starten und warten, bis im\n",
            "           Infobereich \"Docker Desktop läuft\" angezeigt wird.{0}"
        ),
        more_help()
    )
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    let version = run_command(&["docker", "--version"], Duration::from_secs(10));
    if !version.succeeded() {
        return error(
            ID,
            category,
            TITLE,
            "Der Befehl 'docker' wurde über PATH nicht gefunden.",
            Some(install_docker()),
        );
    }
    let info = run_command(&["docker", "info"], Duration::from_secs(20));
    if !info.succeeded() {
        let permission = info.combined().to_lowercase().contains("permission denied");
        let detail = if permission {
            format!(
                concat!(
                    "Docker-Befehlszeile gefunden ({}), aber der Zugriff auf den ",
                    "Daemon-Socket wurde verweigert."
                ),
                version.stdout
            )
        } else {
            format!(
                concat!(
                    "Docker-Befehlszeile gefunden ({}), aber der Daemon ist nicht ",
                    "erreichbar:\n{}"
                ),
                version.stdout,
                info.combined()
            )
        };
        return error(
            ID,
            category,
            TITLE,
            detail,
            Some(if permission {
                docker_permission()
            } else {
                start_docker()
            }),
        );
    }
    let server = info
        .stdout
        .lines()
        .find_map(|line| line.strip_prefix("Server Version:").map(str::trim))
        .unwrap_or("unbekannt");
    ok(
        ID,
        category,
        TITLE,
        format!("Docker-Daemon aktiv, Serverversion: {server}"),
    )
}
