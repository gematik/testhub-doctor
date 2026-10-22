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

use crate::checks::support::more_help;
use crate::results::{error, ok, warn};
use crate::{CheckCategory, CheckResult};

pub(crate) const ID: &str = "smcb-cert";
pub(crate) const TITLE: &str = "SMC-B-Zertifikat vorhanden";

fn remediation() -> String {
    format!(
        concat!(
            "Das SMC-B-Zertifikat wird zum Starten von ZETA Guard benötigt.\n",
            "  1. Ein Test-SMC-B-Zertifikat über das gematik-Anfrageportal beantragen:\n",
            "       https://service.gematik.de/servicedesk/customer/portal/37/create/198\n",
            "     (Die ZIP-Datei wird per E-Mail zugestellt; dies kann einige Tage dauern.)\n",
            "  2. In der ZIP-Datei die .p12-Datei suchen, deren Name 'AUT_E256_' enthält.\n",
            "  3. Datei entpacken und in 'smcb_private.p12' umbenennen.\n",
            "  4. Hier ablegen: infra/docker/backend/zeta/smcb-private/smcb_private.p12\n",
            "Ohne diese Datei starten die ZETA-Container wiederholt neu.{0}"
        ),
        more_help()
    )
}

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let path = root.join("infra/docker/backend/zeta/smcb-private/smcb_private.p12");
    if !path.exists() {
        return error(
            ID,
            category,
            TITLE,
            format!("{} nicht gefunden.", path.display()),
            Some(remediation()),
        );
    }
    match fs::metadata(&path) {
        Ok(metadata) if metadata.len() == 0 => error(
            ID,
            category,
            TITLE,
            format!("{} ist vorhanden, aber leer.", path.display()),
            Some(remediation()),
        ),
        Ok(_) => ok(ID, category, TITLE, path.display().to_string()),
        Err(error) => warn(
            ID,
            category,
            TITLE,
            format!("{} kann nicht gelesen werden: {error}", path.display()),
            Some(remediation()),
        ),
    }
}
