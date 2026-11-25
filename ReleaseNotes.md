<img align="right" width="250" height="47" src="images/gematik_logo_flag_with_background.png"/><br/>

# Release Notes Testhub Doctor

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#unreleased">Unreleased</a></li>
    <li><a href="#release-101">Release 1.0.1</a></li>
    <li><a href="#release-100">Release 1.0.0</a></li>
  </ol>
</details>

## Release 1.0.1

- Fix the SMC-B certificate file check accepting directories and unreadable files. Require a regular, nonempty file with a successful read probe; report metadata and read failures as errors. Symlinks to suitable files remain supported. PKCS#12 contents and certificate validity are not checked.

- Add a for now manually maintained app version in the Cargo manifest and display it in the
  CLI version output and the GUI's About dialog.

- Update Rust dependencies to their latest stable versions, including sysinfo 0.40 and rfd 0.18. Building from source now requires Rust 1.95 or newer.

## Release 1.0.0

- Initial Release
