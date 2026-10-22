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

use std::env;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "network";
pub(crate) const TITLE: &str = "Container-Registry (ghcr.io) erreichbar";
const TROUBLESHOOTING: &str = "https://gematik.github.io/ti2.0-testhub/#_troubleshooting";

fn more_help() -> String {
    format!("\nWeitere Informationen: {TROUBLESHOOTING}")
}

fn remedy() -> String {
    format!(
        concat!(
            "Testhub-Images werden aus öffentlichen Registries geladen. Ohne Zugriff\n",
            "schlagen 'docker compose up' und der Maven-Build fehl. Mögliche Ursachen:\n",
            "  - Keine Internetverbindung\n",
            "  - Unternehmens-Firewall oder TLS-Proxy blockiert ausgehendes HTTPS\n",
            "  - VPN-Verbindung zur Registry erforderlich\n",
            "Bei Verwendung eines Proxyservers:\n",
            "  Docker:  https://docs.docker.com/network/proxy/\n",
            "  Maven:   <proxies> in ~/.m2/settings.xml konfigurieren\n",
            "Hinweis: Bei gesetzten HTTP_PROXY-/HTTPS_PROXY-Variablen muss der Proxy\n",
            "auch in Docker Desktop beziehungsweise für den Docker-Daemon ",
            "konfiguriert sein.{0}"
        ),
        more_help()
    )
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    let addresses = match ("ghcr.io", 443).to_socket_addrs() {
        Ok(addresses) => addresses.collect::<Vec<_>>(),
        Err(error) => return failure(category, error.to_string()),
    };
    let mut last_error = "Die DNS-Abfrage hat keine Adressen zurückgegeben.".to_string();
    let connected = addresses.into_iter().any(|address| {
        match TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
            Ok(_) => true,
            Err(error) => {
                last_error = error.to_string();
                false
            }
        }
    });
    if connected {
        return ok(
            ID,
            category,
            TITLE,
            "TCP-Verbindung zu ghcr.io:443 erfolgreich.",
        );
    }
    failure(category, last_error)
}

fn failure(category: CheckCategory, cause: String) -> CheckResult {
    let detail = format!("ghcr.io:443 ist direkt nicht erreichbar – {cause}");
    let proxy = env::var("HTTPS_PROXY")
        .or_else(|_| env::var("https_proxy"))
        .or_else(|_| env::var("HTTP_PROXY"))
        .or_else(|_| env::var("http_proxy"))
        .ok();
    if let Some(proxy) = proxy {
        warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "{}\n",
                    "Ein Proxy ist konfiguriert ({}); der Zugriff auf die ",
                    "Registry könnte darüber funktionieren."
                ),
                detail, proxy
            ),
            Some(remedy()),
        )
    } else {
        error(ID, category, TITLE, detail, Some(remedy()))
    }
}
