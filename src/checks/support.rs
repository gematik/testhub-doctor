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

const TROUBLESHOOTING: &str = "https://gematik.github.io/ti2.0-testhub/#_troubleshooting";

pub(crate) fn more_help() -> String {
    format!("\nWeitere Informationen: {TROUBLESHOOTING}")
}

pub(crate) fn project_hint() -> String {
    concat!(
        "testhub-doctor im Root-Verzeichnis des TI-2.0-Testhub-Projekts ausführen\n",
        "oder den Pfad explizit angeben:  testhub-doctor check --project-root <path>\n",
        "Falls Dateien tatsächlich fehlen, können sie wiederhergestellt werden:\n",
        "  git status          # Änderungen anzeigen\n",
        "  git checkout -- .   # gelöschte versionierte Dateien wiederherstellen\n",
        "Alternativ das Repository erneut klonen: ",
        "https://github.com/gematik/ti2.0-testhub.git"
    )
    .into()
}
