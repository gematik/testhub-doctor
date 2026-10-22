<img align="right" width="250" height="47" src="images/gematik_logo_flag_with_background.png"/><br/>
# Testhub Doctor
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about-the-project">About The Project</a></li>
    <li><a href="#release-notes">Release Notes</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#third-party-licenses">Third-party Licenses</a></li>
  </ol>
</details>

## About the Project
Testhub Doctor provides a cross-platform command-line interface and a desktop GUI for checking prerequisites of a separate TI 2.0 Testhub working copy. **The minimum Testhub version supported is 3.18.0**; the Doctor checks the expected file layout, but does not detect or enforce the Testhub version. The application is written in Rust; its Tauri GUI uses a small vanilla HTML/CSS/JavaScript frontend. The checks verify the Java, Maven, Docker, and system requirements of the Testhub itself.

The application checks:

| Check ID | What it checks |
|----------|----------------|
| `java-version` | `java -version` reports Java 21+; warns if `javac -version` fails or `JAVA_HOME` is unset |
| `java-home-path` | `JAVA_HOME` is set and resolves to the same JDK as `java` on `PATH` |
| `maven-wrapper` | The platform's wrapper (`mvnw` on macOS/Linux, `mvnw.cmd` on Windows) exists; warns about missing Unix execute permissions or `.mvn/wrapper/` |
| `docker-daemon` | Docker CLI exists and the Docker daemon is reachable |
| `docker-compose` | `docker compose` v2 is available; version 2.30+ is recommended |
| `compose-file` | `infra/docker/compose-local.yaml` exists |
| `smcb-cert` | The SMC-B certificate exists and is not empty |
| `port-availability` | Numeric `PORT_*` entries in `infra/docker/.env` can be bound as TCP ports on `127.0.0.1`; warns if the file is missing/unreadable or no numeric entries are found |
| `system-resources` | Physical RAM and free disk space meet the Doctor's recommendations |
| `network` | A direct TCP connection to `ghcr.io:443` succeeds; a configured proxy downgrades failure to a warning |
| `windows-wsl` | On Windows, inspects `wsl --status` for WSL 2 and, if `docker info` succeeds, looks for a WSL backend; reports the check as skipped on macOS/Linux |

## Requirements for building from source

- Rust 1.92 or newer to build from source
- Tauri GUI build prerequisites: Microsoft C++ Build Tools and WebView2 on Windows; the Linux development packages listed below; Xcode Command Line Tools on macOS
- No Node.js installation is needed; the Tauri frontend uses plain HTML, CSS, and JavaScript

Java, Maven, Docker, and a Testhub working copy are not required to build the Doctor. To pass the corresponding checks, provide a Java 21+ JDK, Docker, Docker Compose, and a Testhub working copy as described under [Testhub prerequisites](#testhub-prerequisites). Missing prerequisites are reported as findings; they do not prevent the Doctor from running. Neither the CLI nor the GUI needs a Java runtime.

## Build

Build the CLI and Tauri GUI on the operating system where you intend to run them:

```bash
cargo build --release --bin testhub-doctor
cargo build --release -p testhub-doctor-gui
```

The CLI binary is written to `target/release/testhub-doctor` (Windows: `target\release\testhub-doctor.exe`). The GUI binary is `target/release/testhub-doctor-gui` (Windows: `target\release\testhub-doctor-gui.exe`). The CLI has no GUI runtime dependency. Each binary is native to the host OS and architecture. Builds for macOS, Linux, and Windows should be produced and tested on their respective platforms; cross-compilation is not configured by this project.

The GUI is a separate Rust executable in `src-gui/` using Tauri as UI toolkit; its Cargo package is named `testhub-doctor-gui`. It lets users enter or browse to the Testhub working directory, reports results as checks finish, and provides severity chips that jump to the first matching result. Hints for addressing findings ('Problembehebungen') can be expanded on demand.

On Ubuntu/Debian, install the Tauri GUI's system libraries before building:

```bash
sudo apt install build-essential libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev
```

On Windows, install the **Desktop development with C++** workload from Visual Studio Build Tools and the Microsoft Edge WebView2 Runtime.

On macOS, install Xcode Command Line Tools (`xcode-select --install`).

### Run the application from the repository

```bash
cargo run --release --bin testhub-doctor -- --help
cargo run --release -p testhub-doctor-gui
```

Run the build on each target OS to produce its native executable; cross-compilation is not configured. These commands build the executable, not an installer. Tauri uses the operating system's WebView (WebView2 on Windows, WebKitGTK on Linux, and WebKit on macOS).

## CLI usage

By default, running `testhub-doctor` checks the current directory. Set the Testhub working-copy root with `--project-root` (`-p`); it may be an absolute path or a path relative to the current directory. The option works before or after the `check` command. The directory must exist; a directory that is not a Testhub working copy produces findings for missing project files.

`testhub-doctor --help` lists all commands and their options, including the `check` options. Use `testhub-doctor check --help` (or `testhub-doctor help check`) for help focused on checks. **Output and filter options must follow the explicit `check` subcommand**: use `testhub-doctor check --json`, not `testhub-doctor --json`.

The examples below assume the CLI binary is on `PATH`. When running directly from a source build, replace `testhub-doctor` with `./target/release/testhub-doctor` on macOS/Linux.

```bash
testhub-doctor --help
testhub-doctor check --help
testhub-doctor --version
testhub-doctor
testhub-doctor check --project-root /path/to/ti2.0-testhub
testhub-doctor check --project-root /path/to/ti2.0-testhub --json
testhub-doctor --project-root /path/to/ti2.0-testhub check --json
testhub-doctor list-checks
testhub-doctor check --only java-version,java-home-path,docker-daemon --show-remediation
```

PowerShell examples use the same flags and the `.exe` binary:

```powershell
.\target\release\testhub-doctor.exe check --project-root C:\path\to\ti2.0-testhub
```

### Commands and options

| Command / option | Description |
|------------------|-------------|
| *(no command)* | Run all checks |
| `check` | Run checks and accept output/filter options |
| `list-checks` | Print every check ID and title |
| `generate-completion <shell>` | Print a completion script for supported shells (Bash, Elvish, Fish, PowerShell, or Zsh) |
| `help [command]` | Show main help or help for a subcommand |
| `-p`, `--project-root <PATH>` | Testhub working-copy root; defaults to the current directory; usable before or after `check` |
| `-h`, `--help` | Show help for the command |
| `-V`, `--version` | Show the application version |
| `--only <ID,...>` | Run only the listed check IDs; accepts comma-separated IDs and repeated `--only` options; unknown IDs are errors (`check` only) |
| `--json` | Emit machine-readable JSON (`check` only) |
| `-r`, `--show-remediation` | Also print available remedies for warnings; errors always show remedies (`check` only) |
| `--no-color` | Disable ANSI colors (`check` only) |
| `--ascii` | Use ASCII status symbols (`check` only) |

Selected checks run once each in the application's fixed check order, not in the order passed to `--only`. Redirected terminal output automatically disables colors and uses ASCII status symbols. `--json` outputs only JSON on stdout; terminal formatting and remediation-display flags do not change it.

### Exit codes

- `0`: no check reported an error; warnings may still be present
- `1`: at least one check reported an error
- `2`: invalid arguments (including unknown check IDs), or a project root that cannot be resolved to an existing directory

Help, version, check listing, and completion generation exit with `0` without running checks or validating the project root. Invalid arguments or project roots produce diagnostics on stderr, not a JSON report, even with `check --json`.

The JSON output contains a `summary` (`passed`, `warnings`, `errors`, `skipped`, `status`) and a `checks` array. `status` is `ERROR` if any check fails, `WARN` if there are only warnings, and `OK` otherwise; skipped checks do not count as passed or affect the status. Each check contains `id`, `category`, `title`, `severity`, `detail`, and `remediation` (a string or `null`). Severity values are `OK`, `WARN`, `ERROR`, and `SKIPPED`; categories are `JAVA`, `DOCKER`, `PROJECT`, `SYSTEM`, `NETWORK`, and `WINDOWS`. Available remediation text is included in JSON regardless of `--show-remediation`.

## Testhub prerequisites

### Java 21+

The Doctor's Java checks apply to the **Testhub build**, not to running the native Doctor binary. Install a JDK and ensure both `java` and `javac` are on `PATH`; set `JAVA_HOME` to the JDK root.

- macOS: `brew install --cask temurin@21`
- Ubuntu/Debian: `sudo apt install openjdk-21-jdk`
- Windows: `winget install EclipseAdoptium.Temurin.21.JDK`

### Docker and Compose

Install and start Docker, and make sure `docker compose version` reports Compose v2. On Linux, the user running the checks needs permission to access the Docker daemon. On Windows, enable WSL 2 and Docker Desktop's WSL 2 based engine.

The Testhub Compose configuration uses the Compose `include` feature, which requires Docker Compose 2.20 or newer. The Doctor invokes `docker compose version` and warns when it recognizes a version below 2.30; it does not parse or validate the Compose file or enforce the `include` feature's minimum version. The 2.30 threshold is a Doctor recommendation, not an officially stated Testhub minimum. The legacy `docker-compose` command is not used as a fallback.

### Doctor resource recommendations

The Doctor recommends 8 GiB of physical RAM and 10 GiB of free disk space for the full stack. These are heuristic recommendations made by the Doctor, not official Testhub minimum requirements. RAM below 6 GiB produces an error; RAM from 6 GiB to below 8 GiB produces a warning. Free disk space below 10 GiB on either the project or user-home filesystem produces a warning. If a resource measurement fails, it produces a warning. The combined check reports the finding with the highest severity; RAM takes precedence when severities are equal. On macOS and Windows, also check Docker Desktop's own memory and disk-image limits, which may differ from the host resources measured here.

The project checks expect these paths inside the Testhub working copy:

```text
infra/docker/compose-local.yaml
infra/docker/.env
infra/docker/backend/zeta/smcb-private/smcb_private.p12
```

The `.env` file's `PORT_*` entries are tested by attempting to bind each port on the IPv4 loopback interface. Only plain numeric values are parsed; quoting, variable expansion, inline comments, and environment overrides are not evaluated. The check does not test UDP, IPv6, or container-side ports. The certificate check verifies existence and nonzero size, not certificate validity or its password.

Network connectivity is tested by DNS resolution and a direct TCP connection to `ghcr.io:443`, not by TLS, authentication, or an image pull. The presence of `HTTPS_PROXY`, `https_proxy`, `HTTP_PROXY`, or `http_proxy` changes a failed probe from an error to a warning; it does not tunnel the Doctor's TCP probe through the proxy or verify Docker's proxy configuration.

### Release Notes
See [ReleaseNotes.md](./ReleaseNotes.md) for all information regarding the (newest) releases.

## License

Copyright 2026 gematik GmbH

Apache License, Version 2.0

See the [LICENSE](./LICENSE) for the specific language governing permissions and limitations under the License

## Third-party licenses

Rust crate licenses, including their license texts, are listed in [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) and are also available from the desktop app's **Open-Source-Lizenzen** button. The generated report covers direct dependencies of the workspace packages and the direct build dependency `tauri-build`; it intentionally omits transitive dependencies (including Tauri's dependency tree) and development dependencies. Operating-system-provided components such as WebView runtimes are outside its scope.

After adding, changing, or removing a dependency in a workspace manifest, regenerate both reports manually with `cargo-about 0.9.2`. The renderer is a Rust binary in this workspace. The update script reads the locked workspace graph and writes `THIRD_PARTY_LICENSES.md` and `frontend/third_party_licenses.html`; it does not run automatically when Cargo files change. Commit the regenerated reports with the dependency change. Identical license texts are grouped by SPDX identifier while package-specific copyright notices and materially different license texts are retained:

```bash
cargo install cargo-about --version 0.9.2 --locked --features cli
bash scripts/update-licenses.sh
```

## Additional Notes and Disclaimer from gematik GmbH

1. Copyright notice: Each published work result is accompanied by an explicit statement of the license conditions for
   use. These are regularly typical conditions in connection with open source or free software. Programs
   described/provided/linked here are free software, unless otherwise stated.
2. Permission notice: Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
   associated documentation files (the "Software"), to deal in the Software without restriction, including without
   limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the
   Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
    1. The copyright notice (Item 1) and the permission notice (Item 2) shall be included in all copies or substantial
       portions of the Software.
    2. The software is provided "as is" without warranty of any kind, either express or implied, including, but not
       limited to, the warranties of fitness for a particular purpose, merchantability, and/or non-infringement. The
       authors or copyright holders shall not be liable in any manner whatsoever for any damages or other claims arising
       from, out of or in connection with the software or the use or other dealings with the software, whether in an
       action of contract, tort, or otherwise.
    3. The software is the result of research and development activities, therefore not necessarily quality assured and
       without the character of a liable product. For this reason, gematik does not provide any support or other user
       assistance (unless otherwise stated in individual cases and without justification of a legal obligation).
       Furthermore, there is no claim to further development and adaptation of the results to a more current state of
       the art.
3. Gematik may remove published results temporarily or permanently from the place of publication at any time without
   prior notice or justification.
4. Please note: Parts of this code may have been generated using AI-supported technology. Please take this into account,
   especially when troubleshooting, for security analyses and possible adjustments.
