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

use super::support::project_hint;
use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "maven-wrapper";
pub(crate) const TITLE: &str = "Maven-Wrapper vorhanden";

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let wrapper = root.join(if cfg!(windows) { "mvnw.cmd" } else { "mvnw" });
    if !wrapper.exists() {
        return error(
            ID,
            category,
            TITLE,
            format!("{} nicht gefunden.", wrapper.display()),
            Some(project_hint()),
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&wrapper)
            .map(|metadata| metadata.permissions().mode() & 0o111 == 0)
            .unwrap_or(true)
        {
            return warn(
                ID,
                category,
                TITLE,
                format!(
                    "{} ist vorhanden, aber nicht ausführbar.",
                    wrapper.display()
                ),
                Some(format!(
                    concat!("Den Wrapper ausführbar machen:\n", "  chmod +x {}"),
                    wrapper.display()
                )),
            );
        }
    }
    if !root.join(".mvn/wrapper").is_dir() {
        return warn(
            ID,
            category,
            TITLE,
            concat!(
                "Das Verzeichnis .mvn/wrapper/ fehlt – der Wrapper kann Maven ",
                "möglicherweise nicht bereitstellen."
            ),
            Some(
                concat!(
                    "Aus dem Repository wiederherstellen:\n",
                    "  git checkout -- .mvn/"
                )
                .into(),
            ),
        );
    }
    ok(
        ID,
        category,
        TITLE,
        format!("{} ist vorhanden und ausführbar.", wrapper.display()),
    )
}
