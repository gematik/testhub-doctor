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

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use wait_timeout::ChildExt;

pub(crate) struct ProcessOutput {
    pub(crate) code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl ProcessOutput {
    pub(crate) fn succeeded(&self) -> bool {
        self.code == 0
    }

    pub(crate) fn combined(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
            .trim()
            .to_string()
    }
}

pub(crate) fn run_command(args: &[&str], timeout: Duration) -> ProcessOutput {
    let Some((program, rest)) = args.split_first() else {
        return ProcessOutput {
            code: -1,
            stdout: String::new(),
            stderr: "Leerer Befehl".into(),
        };
    };
    let mut child = match Command::new(program)
        .args(rest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return ProcessOutput {
                code: -1,
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    };
    let stdout = child.stdout.take().map(read_async);
    let stderr = child.stderr.take().map(read_async);
    let status = match child.wait_timeout(timeout) {
        Ok(Some(status)) => status,
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return ProcessOutput {
                code: -1,
                stdout: stdout
                    .and_then(|reader| reader.join().ok())
                    .unwrap_or_default(),
                stderr: format!("Zeitüberschreitung nach {} s", timeout.as_secs()),
            };
        }
        Err(error) => {
            return ProcessOutput {
                code: -1,
                stdout: String::new(),
                stderr: error.to_string(),
            };
        }
    };
    ProcessOutput {
        code: status.code().unwrap_or(-1),
        stdout: stdout
            .and_then(|reader| reader.join().ok())
            .unwrap_or_default(),
        stderr: stderr
            .and_then(|reader| reader.join().ok())
            .unwrap_or_default(),
    }
}

fn read_async<R: Read + Send + 'static>(mut reader: R) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = reader.read_to_end(&mut bytes);
        decode_output(&bytes)
    })
}

fn decode_output(bytes: &[u8]) -> String {
    let utf16 = bytes.starts_with(&[0xff, 0xfe])
        || (bytes.len() >= 4
            && bytes
                .iter()
                .skip(1)
                .step_by(2)
                .take(32)
                .filter(|byte| **byte == 0)
                .count()
                > bytes.len().min(64) / 8);
    let text = if utf16 {
        let content = if bytes.starts_with(&[0xff, 0xfe]) {
            &bytes[2..]
        } else {
            bytes
        };
        String::from_utf16_lossy(
            &content
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes(*pair))
                .collect::<Vec<_>>(),
        )
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    };
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::decode_output;

    #[test]
    fn command_output_detects_windows_utf16() {
        assert_eq!(decode_output(&[0xff, 0xfe, b'h', 0, b'i', 0]), "hi");
    }
}
