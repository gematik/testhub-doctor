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
 * #L%
 */

mod checks;
mod process;
mod results;

use serde::Serialize;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Severity {
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "SKIPPED")]
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CheckCategory {
    #[serde(rename = "JAVA")]
    Java,
    #[serde(rename = "DOCKER")]
    Docker,
    #[serde(rename = "PROJECT")]
    Project,
    #[serde(rename = "SYSTEM")]
    System,
    #[serde(rename = "NETWORK")]
    Network,
    #[serde(rename = "WINDOWS")]
    Windows,
}

impl CheckCategory {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Java => "Java / Buildwerkzeuge",
            Self::Docker => "Docker",
            Self::Project => "Projektdateien",
            Self::System => "Systemressourcen",
            Self::Network => "Netzwerk",
            Self::Windows => "Windows / WSL",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckResult {
    #[serde(rename = "id")]
    pub check_id: String,
    pub category: CheckCategory,
    pub title: String,
    pub severity: Severity,
    pub detail: String,
    pub remediation: Option<String>,
}

const CHECKS: [(&str, &str, CheckCategory); 11] = [
    (
        checks::java_version::ID,
        checks::java_version::TITLE,
        CheckCategory::Java,
    ),
    (
        checks::java_home::ID,
        checks::java_home::TITLE,
        CheckCategory::Java,
    ),
    (
        checks::maven_wrapper::ID,
        checks::maven_wrapper::TITLE,
        CheckCategory::Java,
    ),
    (
        checks::docker_daemon::ID,
        checks::docker_daemon::TITLE,
        CheckCategory::Docker,
    ),
    (
        checks::docker_compose::ID,
        checks::docker_compose::TITLE,
        CheckCategory::Docker,
    ),
    (
        checks::compose_file::ID,
        checks::compose_file::TITLE,
        CheckCategory::Project,
    ),
    (
        checks::smcb_cert::ID,
        checks::smcb_cert::TITLE,
        CheckCategory::Project,
    ),
    (
        checks::port_availability::ID,
        checks::port_availability::TITLE,
        CheckCategory::System,
    ),
    (
        checks::system_resources::ID,
        checks::system_resources::TITLE,
        CheckCategory::System,
    ),
    (
        checks::network::ID,
        checks::network::TITLE,
        CheckCategory::Network,
    ),
    (
        checks::windows::ID,
        checks::windows::TITLE,
        CheckCategory::Windows,
    ),
];

pub fn check_ids() -> Vec<(&'static str, &'static str)> {
    CHECKS.iter().map(|(id, title, _)| (*id, *title)).collect()
}

pub fn check_info(id: &str) -> Option<(&'static str, &'static str, CheckCategory)> {
    CHECKS
        .iter()
        .find(|(check_id, _, _)| *check_id == id)
        .map(|(check_id, title, category)| (*check_id, *title, *category))
}

pub fn run_checks(root: &Path, only: &[String]) -> Vec<CheckResult> {
    CHECKS
        .iter()
        .filter(|(id, _, _)| only.is_empty() || only.iter().any(|wanted| wanted == id))
        .map(|(id, _, _)| run_check(id, root).expect("registered check ID is valid"))
        .collect()
}

pub fn run_check(id: &str, root: &Path) -> Option<CheckResult> {
    let (_, _, category) = CHECKS.iter().find(|(check_id, _, _)| *check_id == id)?;
    Some(match id {
        checks::java_version::ID => checks::java_version::check(*category),
        checks::java_home::ID => checks::java_home::check(*category),
        checks::maven_wrapper::ID => checks::maven_wrapper::check(*category, root),
        checks::docker_daemon::ID => checks::docker_daemon::check(*category),
        checks::docker_compose::ID => checks::docker_compose::check(*category),
        checks::compose_file::ID => checks::compose_file::check(*category, root),
        checks::smcb_cert::ID => checks::smcb_cert::check(*category, root),
        checks::port_availability::ID => checks::port_availability::check(*category, root),
        checks::system_resources::ID => checks::system_resources::check(*category, root),
        checks::network::ID => checks::network::check(*category),
        checks::windows::ID => checks::windows::check(*category),
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::{CheckCategory, Severity};
    use crate::results::ok;

    #[test]
    fn serializes_cli_json_field_names_and_severity() {
        let result = ok("network", CheckCategory::Network, "Registry", "ready");
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["id"], "network");
        assert_eq!(json["severity"], "OK");
        assert_eq!(json["category"], "NETWORK");
        assert_eq!(serde_json::to_value(Severity::Skipped).unwrap(), "SKIPPED");
    }
}
