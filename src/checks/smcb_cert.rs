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

use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::checks::support::more_help;
use crate::results::{error, ok};
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
            "Der Pfad muss auf eine reguläre, lesbare und nichtleere Datei zeigen.\n",
            "Ohne diese Datei starten die ZETA-Container wiederholt neu.{0}"
        ),
        more_help()
    )
}

pub(crate) fn check(category: CheckCategory, root: &Path) -> CheckResult {
    let path = root.join("infra/docker/backend/zeta/smcb-private/smcb_private.p12");
    match validate_file(&path) {
        Ok(()) => ok(
            ID,
            category,
            TITLE,
            format!(
                "{} ist eine reguläre, lesbare und nichtleere Datei. \
                 Die Zertifikatsgültigkeit wurde nicht geprüft.",
                path.display()
            ),
        ),
        Err(reason) => error(
            ID,
            category,
            TITLE,
            format!("{} {reason}", path.display()),
            Some(remediation()),
        ),
    }
}

fn validate_file(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            "nicht gefunden.".to_string()
        } else {
            format!("Metadaten können nicht gelesen werden: {error}")
        }
    })?;
    if !metadata.is_file() {
        return Err("ist keine reguläre Datei.".to_string());
    }

    let mut file =
        File::open(path).map_err(|error| format!("kann nicht geöffnet werden: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("Metadaten können nicht gelesen werden: {error}"))?;
    if !metadata.is_file() {
        return Err("ist keine reguläre Datei.".to_string());
    }

    file.read_exact(&mut [0]).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            "ist vorhanden, aber leer.".to_string()
        } else {
            format!("kann nicht gelesen werden: {error}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{ID, check};
    use crate::{CheckCategory, CheckResult, Severity};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Project {
        root: PathBuf,
        cert: PathBuf,
    }

    impl Project {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let root = loop {
                let path = std::env::temp_dir().join(format!(
                    "testhub-doctor-smcb-{}-{}",
                    std::process::id(),
                    NEXT_ID.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&path) {
                    Ok(()) => break path,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("cannot create test directory: {error}"),
                }
            };
            let cert = root.join("infra/docker/backend/zeta/smcb-private/smcb_private.p12");
            let project = Self { root, cert };
            fs::create_dir_all(project.cert.parent().unwrap()).unwrap();
            project
        }

        fn check(&self) -> CheckResult {
            check(CheckCategory::Project, &self.root)
        }

        fn assert_error(&self, detail: &str) {
            let result = self.check();
            assert_eq!(result.check_id, ID);
            assert_eq!(result.category, CheckCategory::Project);
            assert_eq!(result.severity, Severity::Error);
            assert!(result.detail.contains(detail), "{}", result.detail);
            assert!(result.remediation.is_some());
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("cannot remove test directory");
        }
    }

    #[test]
    fn rejects_missing_file() {
        Project::new().assert_error("nicht gefunden");
    }

    #[test]
    fn rejects_empty_directory() {
        let project = Project::new();
        fs::create_dir(&project.cert).unwrap();
        project.assert_error("keine reguläre Datei");
    }

    #[test]
    fn rejects_nonempty_directory() {
        let project = Project::new();
        fs::create_dir(&project.cert).unwrap();
        fs::write(project.cert.join("entry"), b"content").unwrap();
        project.assert_error("keine reguläre Datei");
    }

    #[test]
    fn rejects_empty_file() {
        let project = Project::new();
        fs::write(&project.cert, []).unwrap();
        project.assert_error("leer");
    }

    #[test]
    fn accepts_readable_nonempty_file_without_claiming_certificate_validity() {
        let project = Project::new();
        fs::write(&project.cert, [1]).unwrap();
        let result = project.check();
        assert_eq!(result.check_id, ID);
        assert_eq!(result.category, CheckCategory::Project);
        assert_eq!(result.severity, Severity::Ok);
        assert!(
            result
                .detail
                .contains("reguläre, lesbare und nichtleere Datei")
        );
        assert!(
            result
                .detail
                .contains("Zertifikatsgültigkeit wurde nicht geprüft")
        );
        assert!(result.remediation.is_none());
    }

    #[test]
    fn accepts_arbitrary_content_in_presence_check() {
        let project = Project::new();
        fs::write(&project.cert, b"not a PKCS12 container").unwrap();
        assert_eq!(project.check().severity, Severity::Ok);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_unreadable_file_when_permissions_are_enforced() {
        use std::fs::File;
        use std::os::unix::fs::PermissionsExt;

        let project = Project::new();
        fs::write(&project.cert, b"content").unwrap();
        fs::set_permissions(&project.cert, fs::Permissions::from_mode(0)).unwrap();
        let open_result = File::open(&project.cert);
        fs::set_permissions(&project.cert, fs::Permissions::from_mode(0o600)).unwrap();
        match open_result {
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
                fs::set_permissions(&project.cert, fs::Permissions::from_mode(0)).unwrap();
                let result = project.check();
                fs::set_permissions(&project.cert, fs::Permissions::from_mode(0o600)).unwrap();
                assert_eq!(result.severity, Severity::Error);
                assert!(result.detail.contains("kann nicht geöffnet werden"));
                assert!(result.remediation.is_some());
            }
            Ok(_) => eprintln!("permission test skipped: current user can read mode-000 files"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn accepts_symlink_to_readable_file() {
        let project = Project::new();
        let target = project.root.join("target.p12");
        fs::write(&target, b"content").unwrap();
        std::os::unix::fs::symlink(&target, &project.cert).unwrap();
        assert_eq!(project.check().severity, Severity::Ok);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_to_directory() {
        let project = Project::new();
        std::os::unix::fs::symlink(&project.root, &project.cert).unwrap();
        project.assert_error("keine reguläre Datei");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_broken_symlink() {
        let project = Project::new();
        std::os::unix::fs::symlink(project.root.join("missing.p12"), &project.cert).unwrap();
        project.assert_error("nicht gefunden");
    }

    #[cfg(unix)]
    #[test]
    fn reports_metadata_errors_instead_of_missing_file() {
        let project = Project::new();
        std::os::unix::fs::symlink(&project.cert, &project.cert).unwrap();
        project.assert_error("Metadaten können nicht gelesen werden");
    }
}
