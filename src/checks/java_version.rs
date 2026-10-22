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
use std::time::Duration;

use super::support::more_help;
use crate::process::run_command;
use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "java-version";
pub(crate) const TITLE: &str = "Java 21 oder neuer (JDK) installiert";

pub(crate) fn install_java() -> String {
    format!(
        concat!(
            "Java Development Kit (JDK) 21 oder neuer installieren:\n",
            "  macOS:   brew install --cask temurin@21\n",
            "           – oder mit SDKMAN: sdk install java 21-tem\n",
            "  Ubuntu:  sudo apt install openjdk-21-jdk\n",
            "  Windows: winget install EclipseAdoptium.Temurin.21.JDK\n",
            "           – oder herunterladen unter https://adoptium.net/\n",
            "Nach der Installation müssen 'java' und 'javac' im PATH liegen.\n",
            "JAVA_HOME muss auf das Root-Verzeichnis des JDK verweisen.{0}"
        ),
        more_help()
    )
}

pub(crate) fn set_java_home() -> String {
    concat!(
        "JAVA_HOME auf das Root-Verzeichnis der Java-21-JDK-Installation setzen:\n",
        "  macOS:   export JAVA_HOME=$(/usr/libexec/java_home -v 21)\n",
        "  Linux:   export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64\n",
        "           (für dauerhafte Übernahme in ~/.bashrc oder ~/.zshrc eintragen)\n",
        "  Windows: setx JAVA_HOME \"C:\\Program Files\\Eclipse Adoptium\\jdk-21\"\n",
        "           (anschließend das Terminal neu starten)"
    )
    .into()
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    let output = run_command(&["java", "-version"], Duration::from_secs(10));
    if !output.succeeded() {
        return error(
            ID,
            category,
            TITLE,
            format!(
                "Der Befehl 'java' wurde nicht gefunden oder ist fehlgeschlagen: {}",
                output.combined()
            ),
            Some(install_java()),
        );
    }
    let version = output.combined();
    let Some(major) = parse_java_major(&version) else {
        return error(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "Die Java-Version lässt sich nicht aus folgender Ausgabe ermitteln: ",
                    "{}"
                ),
                version
            ),
            Some(install_java()),
        );
    };
    if major < 21 {
        return error(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "Java {} gefunden; erforderlich ist Java 21 oder neuer.\n",
                    "{}"
                ),
                major, version
            ),
            Some(install_java()),
        );
    }
    if !run_command(&["javac", "-version"], Duration::from_secs(10)).succeeded() {
        return warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "Java {} gefunden, aber 'javac' fehlt – möglicherweise ist nur eine ",
                    "JRE statt eines JDK installiert.\n",
                    "Für den Maven-Build wird ein vollständiges JDK benötigt."
                ),
                major
            ),
            Some(install_java()),
        );
    }
    if let Some(home) = env::var("JAVA_HOME")
        .ok()
        .filter(|home| !home.trim().is_empty())
    {
        ok(
            ID,
            category,
            TITLE,
            format!(concat!("{}\n", "JAVA_HOME={}"), version, home),
        )
    } else {
        warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "{}\n",
                    "JAVA_HOME ist nicht gesetzt – Maven könnte ein anderes JDK verwenden."
                ),
                version
            ),
            Some(set_java_home()),
        )
    }
}

fn parse_java_major(text: &str) -> Option<u32> {
    let marker = text.find("version \"")? + "version \"".len();
    let mut parts = text[marker..].split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    if major == 1 {
        parts
            .next()?
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    } else {
        Some(major)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_java_major;

    #[test]
    fn java_version_parser_handles_common_formats() {
        assert_eq!(parse_java_major("openjdk version \"21.0.2\""), Some(21));
        assert_eq!(parse_java_major("java version \"1.8.0_402\""), Some(8));
        assert_eq!(parse_java_major("no version here"), None);
    }
}
