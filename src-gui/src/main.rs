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

use serde::Serialize;
use std::path::PathBuf;
use tauri::Manager;
use testhub_doctor::{CheckResult, check_ids, check_info};

#[derive(Serialize)]
struct CheckDescriptor {
    id: &'static str,
    title: &'static str,
}

#[tauri::command]
fn list_checks() -> Vec<CheckDescriptor> {
    check_ids()
        .into_iter()
        .map(|(id, title)| CheckDescriptor { id, title })
        .collect()
}

#[tauri::command]
fn default_project_root() -> Result<String, String> {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| format!("Aktuelles Verzeichnis kann nicht gelesen werden: {error}"))
}

#[tauri::command]
fn validate_project_root(path: String) -> Result<String, String> {
    normalize_directory(&path).map(|root| root.to_string_lossy().into_owned())
}

#[tauri::command]
async fn pick_folder(current: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new().set_title("Testhub-Workingcopy auswählen");
        if PathBuf::from(&current).is_dir() {
            dialog = dialog.set_directory(&current);
        }
        dialog
            .pick_folder()
            .map(|path| path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| format!("Ordnerauswahl fehlgeschlagen: {error}"))
}

#[tauri::command]
async fn run_check(check_id: String, project_root: String) -> Result<CheckResult, String> {
    if check_info(&check_id).is_none() {
        return Err(format!("Unbekannter Check: {check_id}"));
    }
    let root = normalize_directory(&project_root)?;

    tauri::async_runtime::spawn_blocking(move || {
        testhub_doctor::run_check(&check_id, &root)
            .ok_or_else(|| format!("Check konnte nicht ausgeführt werden: {check_id}"))
    })
    .await
    .map_err(|error| format!("Check fehlgeschlagen: {error}"))?
}

fn normalize_directory(path: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(path)
        .canonicalize()
        .map_err(|_| format!("Testhub-Verzeichnis nicht gefunden: {path}"))?;
    if !root.is_dir() {
        return Err(format!(
            "Testhub-Verzeichnis nicht gefunden: {}",
            root.display()
        ));
    }
    Ok(root)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            app.get_webview_window("main")
                .expect("main window is configured")
                .set_icon(icon)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_checks,
            default_project_root,
            validate_project_root,
            pick_folder,
            run_check
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Testhub Doctor GUI");
}
