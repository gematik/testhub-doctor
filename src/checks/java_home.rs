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
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::java_version::{install_java, set_java_home};
use crate::process::run_command;
use crate::results::{ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "java-home-path";
pub(crate) const TITLE: &str = "JAVA_HOME gesetzt und mit PATH konsistent";

fn parse_java_home(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("java.home =")
            .map(str::trim)
            .filter(|home| !home.is_empty())
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir().unwrap_or_default().join(path)
        }
    })
}

pub(crate) fn check(category: CheckCategory) -> CheckResult {
    let Some(home) = env::var("JAVA_HOME")
        .ok()
        .filter(|home| !home.trim().is_empty())
    else {
        return warn(
            ID,
            category,
            TITLE,
            "JAVA_HOME ist nicht gesetzt.",
            Some(set_java_home()),
        );
    };
    let output = if cfg!(windows) {
        run_command(&["where.exe", "java"], Duration::from_secs(10))
    } else {
        run_command(&["which", "java"], Duration::from_secs(10))
    };
    if !output.succeeded() || output.stdout.is_empty() {
        return warn(
            ID,
            category,
            TITLE,
            "Der Pfad zum Befehl 'java' lässt sich über PATH nicht ermitteln.",
            Some(install_java()),
        );
    }
    let java = output.stdout.lines().next().unwrap_or_default().trim();
    let resolved_java = if cfg!(target_os = "macos") {
        let settings = run_command(
            &["java", "-XshowSettings:properties", "-version"],
            Duration::from_secs(10),
        );
        let settings_text = settings.combined();
        let Some(runtime_home) = parse_java_home(&settings_text) else {
            return warn(
                ID,
                category,
                TITLE,
                format!(
                    concat!(
                        "Das tatsächlich von 'java' verwendete JDK-Verzeichnis ließ sich nicht ",
                        "ermitteln (PATH-Eintrag: {})."
                    ),
                    java
                ),
                Some(install_java()),
            );
        };
        runtime_home.to_string()
    } else {
        java.to_string()
    };
    if !normalize_path(Path::new(&resolved_java)).starts_with(normalize_path(Path::new(&home))) {
        let remediation = concat!(
            "Sicherstellen, dass JAVA_HOME und PATH auf dasselbe JDK verweisen:\n",
            "  macOS/Linux: 'export PATH=\"$JAVA_HOME/bin:$PATH\"' in ~/.bashrc oder ",
            "~/.zshrc eintragen\n",
            "  Windows:     %JAVA_HOME%\\bin am Anfang der PATH-Systemvariablen eintragen,\n",
            "               vor anderen Java-Einträgen (z. B. Oracle javapath)."
        );
        return warn(
            ID,
            category,
            TITLE,
            format!(
                concat!(
                    "JAVA_HOME={}\n",
                    "java in PATH={}\n",
                    "Die Pfade verweisen auf unterschiedliche JDKs."
                ),
                home, java
            ),
            Some(remediation.into()),
        );
    }
    ok(
        ID,
        category,
        TITLE,
        format!("JAVA_HOME={home}  →  {resolved_java}"),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_java_home;

    #[test]
    fn java_home_parser_reads_runtime_property() {
        assert_eq!(
            parse_java_home(concat!(
                "    java.home = ",
                "/Library/Java/JavaVirtualMachines/jdk-21/Contents/Home\n"
            )),
            Some("/Library/Java/JavaVirtualMachines/jdk-21/Contents/Home")
        );
        assert_eq!(parse_java_home("no java home property"), None);
    }
}
