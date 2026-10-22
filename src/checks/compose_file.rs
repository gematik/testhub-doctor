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

use std::path::Path;

use crate::checks::support::project_hint;
use crate::results::{error, ok};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "compose-file";
pub(crate) const TITLE: &str = "Compose-Datei vorhanden";

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let path = root.join("infra/docker/compose-local.yaml");
    if path.exists() {
        ok(ID, category, TITLE, path.display().to_string())
    } else {
        error(
            ID,
            category,
            TITLE,
            format!("{} nicht gefunden.", path.display()),
            Some(project_hint()),
        )
    }
}
