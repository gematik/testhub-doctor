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

use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Package {
    name: String,
    version: String,
    declared_license: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Notice {
    name: String,
    version: String,
    text: String,
}

#[derive(Debug)]
struct LicenseGroup {
    id: String,
    name: String,
    text: String,
    packages: BTreeSet<Package>,
    notices: BTreeSet<Notice>,
    variant: usize,
    anchor: String,
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value
        .get(key)
        .ok_or_else(|| format!("missing JSON field `{key}`").into())
}

fn string_field(value: &Value, key: &str) -> Result<String> {
    field(value, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("JSON field `{key}` must be a string").into())
}

fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    field(value, key)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("JSON field `{key}` must be an array").into())
}

fn optional_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or("Unknown")
        .to_owned()
}

fn strip_comment_prefix(line: &str) -> &str {
    let mut text = line.trim_start();
    if let Some(rest) = text.strip_prefix("/*") {
        text = rest.trim_start_matches('*');
    } else if let Some(rest) = text
        .strip_prefix("//")
        .or_else(|| text.strip_prefix('*'))
        .or_else(|| text.strip_prefix('#'))
    {
        text = rest;
    }
    text.trim_start()
}

fn is_copyright_notice(line: &str) -> bool {
    let text = strip_comment_prefix(line).trim();
    if text.starts_with('©') {
        return true;
    }
    let Some(owner) = text
        .get(..9)
        .filter(|prefix| prefix.eq_ignore_ascii_case("copyright"))
        .map(|_| text[9..].trim_start())
    else {
        return false;
    };

    let owner_lower = owner.to_ascii_lowercase();
    let excluded = ["notice", "license", "holder", "holders", "and permission"];
    if excluded.iter().any(|prefix| {
        owner_lower
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || !rest.as_bytes()[0].is_ascii_alphanumeric())
    }) {
        return false;
    }
    if [
        "[yyyy]",
        "[year]",
        "[name of copyright owner]",
        "{yyyy}",
        "{year}",
    ]
    .iter()
    .any(|placeholder| owner_lower.contains(placeholder))
    {
        return false;
    }
    let year_candidate = owner_lower
        .strip_prefix("(c)")
        .map(str::trim_start)
        .unwrap_or(&owner_lower);
    let bytes = year_candidate.as_bytes();
    if bytes.len() >= 4 && bytes[..4].iter().all(u8::is_ascii_digit) {
        return true;
    }
    if let Some(rest) = year_candidate
        .strip_prefix('[')
        .or_else(|| year_candidate.strip_prefix('{'))
        .or_else(|| year_candidate.strip_prefix('<'))
    {
        return rest.starts_with("yyyy")
            || rest.starts_with("year")
            || rest
                .as_bytes()
                .get(..4)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_digit));
    }
    false
}

fn split_notices(text: &str) -> (String, Vec<String>) {
    let mut body = Vec::new();
    let mut notices: Vec<String> = Vec::new();
    let mut in_notice = false;
    for line in text.lines() {
        if is_copyright_notice(line) {
            notices.push(line.trim().to_owned());
            in_notice = true;
        } else if in_notice && !line.trim().is_empty() {
            if let Some(notice) = notices.last_mut() {
                notice.push(' ');
                notice.push_str(line.trim());
            }
        } else {
            body.push(line.trim_end());
            in_notice = false;
        }
    }
    (body.join("\n").trim().to_owned(), notices)
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if separator && !result.is_empty() {
                result.push('-');
            }
            result.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    result
}

fn collect_groups(data: &Value) -> Result<Vec<LicenseGroup>> {
    let mut groups: Vec<LicenseGroup> = Vec::new();
    for entry in array_field(data, "licenses")? {
        let id = string_field(entry, "id")?;
        let name = string_field(entry, "name")?;
        let source_text = string_field(entry, "text")?;
        let (text, notices) = split_notices(&source_text);
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let group_index = groups.iter().position(|group| {
            group.id == id
                && group.text.split_whitespace().collect::<Vec<_>>().join(" ") == normalized
        });
        let index = group_index.unwrap_or_else(|| {
            groups.push(LicenseGroup {
                id: id.clone(),
                name,
                text,
                packages: BTreeSet::new(),
                notices: BTreeSet::new(),
                variant: 0,
                anchor: String::new(),
            });
            groups.len() - 1
        });

        for reference in array_field(entry, "used_by")? {
            let crate_data = field(reference, "crate")?;
            let package = Package {
                name: string_field(crate_data, "name")?,
                version: string_field(crate_data, "version")?,
                declared_license: optional_string(crate_data, "license"),
            };
            groups[index].packages.insert(package.clone());
            for notice in &notices {
                groups[index].notices.insert(Notice {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    text: notice.clone(),
                });
            }
        }
    }

    let mut variant_counts: HashMap<String, usize> = HashMap::new();
    for group in &mut groups {
        let count = variant_counts.entry(group.id.clone()).or_default();
        *count += 1;
        group.variant = *count;
        group.anchor = format!("license-{}-{}", slug(&group.id), group.variant);
    }
    Ok(groups)
}

fn markdown_report(data: &Value, groups: &[LicenseGroup]) -> Result<String> {
    let mut lines = vec![
        "# Third-party licenses".to_owned(),
        String::new(),
        concat!(
            "This file is generated from the locked Cargo workspace manifests. It includes local ",
            "workspace packages and their direct dependencies, including build dependencies, ",
            "while transitive and development dependencies are intentionally omitted. Regenerate ",
            "it with `bash scripts/update-licenses.sh` after changing workspace manifests or ",
            "`Cargo.lock`. ",
            "Operating-system-provided components (such as WebView runtimes) are not Cargo ",
            "dependencies and are not covered here."
        )
        .to_owned(),
        String::new(),
        "## License overview".to_owned(),
        String::new(),
        concat!(
            "Counts can overlap: a package declaring alternatives such as `MIT OR Apache-2.0` is ",
            "counted under both licenses. These values are not additive."
        )
        .to_owned(),
        String::new(),
        "| SPDX identifier | License | Package uses |".to_owned(),
        "| --- | --- | ---: |".to_owned(),
    ];
    for item in array_field(data, "overview")? {
        lines.push(format!(
            "| `{}` | {} | {} |",
            string_field(item, "id")?,
            string_field(item, "name")?,
            field(item, "count")?
        ));
    }
    lines.extend([
        String::new(),
        "## Packages".to_owned(),
        String::new(),
        "| Package | Version | Declared license |".to_owned(),
        "| --- | --- | --- |".to_owned(),
    ]);
    for item in array_field(data, "crates")? {
        let crate_data = field(item, "package")?;
        lines.push(format!(
            "| `{}` | `{}` | `{}` |",
            string_field(crate_data, "name")?,
            string_field(crate_data, "version")?,
            optional_string(crate_data, "license")
        ));
    }
    lines.extend([
        String::new(),
        "## License texts and attributions".to_owned(),
        String::new(),
        concat!(
            "Identical license text is shown once per SPDX identifier. Package-specific copyright ",
            "notices are retained below; materially different license texts are shown as separate ",
            "variants."
        )
        .to_owned(),
    ]);
    for group in groups {
        lines.extend([
            String::new(),
            format!(
                "### {} (`{}`) — text variant {}",
                group.name, group.id, group.variant
            ),
            String::new(),
            "Packages:".to_owned(),
        ]);
        for package in &group.packages {
            lines.push(format!(
                "- `{}` {} — declared license: `{}`",
                package.name, package.version, package.declared_license
            ));
        }
        if !group.notices.is_empty() {
            lines.extend([String::new(), "Copyright notices:".to_owned()]);
            for notice in &group.notices {
                lines.push(format!(
                    "- `{}` {}: {}",
                    notice.name, notice.version, notice.text
                ));
            }
        }
        lines.extend([
            String::new(),
            "<pre>".to_owned(),
            escape_html(&group.text, false),
            "</pre>".to_owned(),
        ]);
    }
    Ok(format!("{}\n", lines.join("\n").trim_end()))
}

fn escape_html(value: &str, quote: bool) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' if quote => escaped.push_str("&quot;"),
            '\'' if quote => escaped.push_str("&#x27;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn html_report(data: &Value, groups: &[LicenseGroup]) -> Result<String> {
    let mut html = String::new();
    let mut package_rows = Vec::new();
    for item in array_field(data, "crates")? {
        let crate_data = field(item, "package")?;
        package_rows.push(format!(
            "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
            escape_html(&string_field(crate_data, "name")?, true),
            escape_html(&string_field(crate_data, "version")?, true),
            escape_html(&optional_string(crate_data, "license"), true)
        ));
    }

    let mut first_group_by_license = HashMap::new();
    for group in groups {
        first_group_by_license
            .entry(group.id.as_str())
            .or_insert(group.anchor.as_str());
    }
    let mut overview = Vec::new();
    for item in array_field(data, "overview")? {
        let id = string_field(item, "id")?;
        let name = escape_html(&string_field(item, "name")?, true);
        let count = field(item, "count")?;
        if let Some(anchor) = first_group_by_license.get(id.as_str()) {
            overview.push(format!(
                "<li><a href=\"#{}\">{} ({})</a></li>",
                escape_html(anchor, true),
                name,
                count
            ));
        } else {
            overview.push(format!("<li>{name} ({count})</li>"));
        }
    }

    let mut license_sections = Vec::new();
    for group in groups {
        let packages = group
            .packages
            .iter()
            .map(|package| {
                format!(
                    "<li><code>{}</code> {} — declared license: <code>{}</code></li>",
                    escape_html(&package.name, true),
                    escape_html(&package.version, true),
                    escape_html(&package.declared_license, true)
                )
            })
            .collect::<String>();
        let notices = if group.notices.is_empty() {
            String::new()
        } else {
            let notice_items = group
                .notices
                .iter()
                .map(|notice| {
                    format!(
                        "<li><code>{}</code> {}: {}</li>",
                        escape_html(&notice.name, true),
                        escape_html(&notice.version, true),
                        escape_html(&notice.text, true)
                    )
                })
                .collect::<String>();
            format!("<h4>Copyright notices</h4><ul>{notice_items}</ul>")
        };
        license_sections.push(format!(
            concat!(
                "<details class=\"license-entry\" id=\"{}\"><summary>{} ",
                "(<code>{}</code>) — text variant {}</summary><h4>Packages</h4>",
                "<ul>{}</ul>{}<pre><code>{}</code></pre></details>"
            ),
            escape_html(&group.anchor, true),
            escape_html(&group.name, true),
            escape_html(&group.id, true),
            group.variant,
            packages,
            notices,
            escape_html(&group.text, true)
        ));
    }
    let count = array_field(data, "crates")?.len();
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("  <title>Third-party licenses — Testhub Doctor</title>\n  <style>\n");
    html.push_str(
        r#"    :root { color-scheme: dark; font: 14px/1.5 system-ui, sans-serif; color: #fff;
      background: #101820; scrollbar-gutter: stable; scrollbar-width: auto;
      scrollbar-color: #557782 #192630; }
    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }
    body { margin: 0; padding: 0 20px 24px; }
    a { color: #6dd8d0; }
    h1, h2, h3 { color: #6dd8d0; }
    .muted { color: #adc1c9; }
    .sticky-header { position: sticky; z-index: 2; top: 0; display: flex;
      align-items: center; gap: 14px; margin: 0 -20px 16px; padding: 12px 20px;
      border-bottom: 1px solid #36505a; background: #101820; }
    .title-block { min-width: 0; flex: 1; }
    .title-block h1 { margin: 0; font-size: 20px; }
    .title-block p { margin: 2px 0 0; color: #adc1c9; font-size: 12px; }
    .package-filter { width: min(340px, 42%); padding: 8px 10px; color: white;
      border: 1px solid #36505a; border-radius: 7px; background: #192630; }
    .panel { margin: 12px 0; padding: 14px 16px; border-radius: 10px;
      background: #192630; }
    .overview { display: flex; flex-wrap: wrap; gap: 8px; padding: 0; list-style: none; }
    .overview a { display: block; padding: 6px 10px; border-radius: 8px;
      background: #253742; text-decoration: none; }
    .package-list { max-height: min(56vh, 520px); overflow: auto; border-radius: 6px;
      scrollbar-gutter: stable; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 7px 9px; border-bottom: 1px solid #36505a; text-align: left;
      vertical-align: top; overflow-wrap: anywhere; }
    th { position: sticky; top: 0; z-index: 1; color: #adc1c9; background: #192630; }
    details { margin: 8px 0; padding: 10px 12px; border-radius: 8px; background: #192630; }
    summary { color: #6dd8d0; cursor: pointer; font-weight: 600; }
    .license-entry pre { max-height: 44vh; overflow: auto; padding: 12px; border-radius: 6px;
      background: #101820; color: #e6edf0; white-space: pre-wrap; overflow-wrap: anywhere; }
    code { font-family: ui-monospace, monospace; overflow-wrap: anywhere; }
    .scrollable { scrollbar-width: auto; scrollbar-color: #557782 #192630; }
    .scrollable::-webkit-scrollbar { width: 14px; height: 14px; }
    .scrollable::-webkit-scrollbar-track { border-radius: 7px; background: #192630; }
    .scrollable::-webkit-scrollbar-thumb { min-height: 42px; border: 3px solid #192630;
      border-radius: 7px; background: #557782; }
    .scrollable::-webkit-scrollbar-thumb:hover { background: #6dd8d0; }
    ::-webkit-scrollbar { width: 14px; height: 14px; }
    ::-webkit-scrollbar-track { border-radius: 7px; background: #192630; }
    ::-webkit-scrollbar-thumb { min-height: 42px; border: 3px solid #101820;
      border-radius: 7px; background: #557782; }
    ::-webkit-scrollbar-thumb:hover { background: #6dd8d0; }
    .filter-status { min-height: 1.5em; margin: 0 0 8px; color: #adc1c9; font-size: 12px; }
    @media (max-width: 680px) { .sticky-header { flex-wrap: wrap; }
      .title-block { flex-basis: calc(100% - 70px); } .package-filter { width: 100%; } }
"#,
    );
    html.push_str("  </style>\n</head>\n<body>\n");
    writeln!(
        html,
        concat!(
            "  <header class=\"sticky-header\"><div class=\"title-block\">",
            "<h1>Third-party licenses</h1><p>{} Cargo packages; ",
            "identical license texts are grouped</p></div>"
        ),
        count
    )?;
    html.push_str(
        "    <input id=\"package-filter\" class=\"package-filter\" type=\"search\" \
         placeholder=\"Filter packages…\" aria-label=\"Filter packages\">\n  </header>\n",
    );
    html.push_str(concat!(
        "  <p class=\"muted\">This generated report covers local workspace packages and ",
        "their direct Cargo dependencies, including build dependencies. Transitive and ",
        "development dependencies are intentionally omitted. Copyright notices are retained ",
        "separately when identical license text is grouped. Operating-system-provided ",
        "components such as WebView runtimes are not included.</p>\n"
    ));
    html.push_str(concat!(
        "  <section class=\"panel\"><h2>License overview</h2>\n    ",
        "<p class=\"muted\">Counts can overlap: a package declaring alternatives such ",
        "as <code>MIT OR Apache-2.0</code> is counted under both licenses. These values are ",
        "not additive.</p>\n    <ul class=\"overview\">"
    ));
    html.push_str(&overview.join(""));
    html.push_str("</ul>\n  </section>\n");
    write!(
        html,
        "  <details id=\"packages\" class=\"panel\"><summary>Packages ({count})"
    )?;
    html.push_str(concat!(
        "</summary>\n    <p id=\"filter-status\" class=\"filter-status\">Search packages ",
        "by name, version, or declared license.</p>\n    ",
        "<div class=\"package-list scrollable\"><table>\n",
        "<thead><tr><th>Package</th><th>Version</th><th>Declared license</th></tr></thead>\n",
        "<tbody>"
    ));
    html.push_str(&package_rows.join(""));
    html.push_str("</tbody>\n    </table></div>\n  </details>\n");
    html.push_str(concat!(
        "  <section><h2>License texts and attributions</h2>\n    ",
        "<p class=\"muted\">Identical license text is shown once per SPDX identifier. ",
        "Package-specific copyright notices are retained below; materially different ",
        "license texts are shown as separate variants.</p>\n    "
    ));
    html.push_str(&license_sections.join("\n    "));
    html.push_str(concat!(
        "\n  </section>\n  <script>\n",
        "    const filter = document.querySelector(\"#package-filter\");\n",
        "    const packageDetails = document.querySelector(\"#packages\");\n",
        "    const rows = [...document.querySelectorAll(\"tbody tr\")];\n",
        "    const status = document.querySelector(\"#filter-status\");\n",
        "    filter.addEventListener(\"input\", () => {\n",
        "      const query = filter.value.trim().toLowerCase();\n",
        "      packageDetails.open = query.length > 0 || packageDetails.open;\n",
        "      let visible = 0;\n",
        "      for (const row of rows) {\n",
        "        const matches = row.textContent.toLowerCase().includes(query);\n",
        "        row.hidden = !matches;\n",
        "        if (matches) visible += 1;\n",
        "      }\n",
        "      status.textContent = query ? `${visible} of ${rows.length} packages` : ",
        "`Showing all ${rows.length} packages`;\n",
        "    });\n",
        "    function openLicenseAnchor() {\n",
        "      if (!location.hash) return;\n",
        "      const target = document.getElementById(\n",
        "        decodeURIComponent(location.hash.slice(1))\n",
        "      );\n",
        "      if (target instanceof HTMLDetailsElement) target.open = true;\n",
        "    }\n",
        "    window.addEventListener(\"hashchange\", openLicenseAnchor);\n",
        "    openLicenseAnchor();\n",
        "  </script>\n</body>\n</html>\n"
    ));
    Ok(html)
}

fn arguments() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let mut input = None;
    let mut markdown = None;
    let mut html = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value after {argument}"))?;
        match argument.as_str() {
            "--input" => input = Some(PathBuf::from(value)),
            "--markdown" => markdown = Some(PathBuf::from(value)),
            "--html" => html = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    Ok((
        input.ok_or("required argument missing: --input")?,
        markdown.ok_or("required argument missing: --markdown")?,
        html.ok_or("required argument missing: --html")?,
    ))
}

fn run() -> Result<()> {
    let (input, markdown, html) = arguments()?;
    let data: Value = serde_json::from_slice(&fs::read(input)?)?;
    let groups = collect_groups(&data)?;
    fs::write(markdown, markdown_report(&data, &groups)?)?;
    fs::write(html, html_report(&data, &groups)?)?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("render-licenses: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_groups, escape_html, is_copyright_notice, split_notices};
    use serde_json::json;

    #[test]
    fn groups_identical_text_and_keeps_distinct_copyright_notices() {
        let data = json!({"licenses": [
            {"id": "MIT", "name": "MIT License", "text": "MIT terms\nCopyright 2024 Alice\n",
             "used_by": [{"crate": {"name": "a", "version": "1", "license": "MIT"}}]},
            {"id": "MIT", "name": "MIT License", "text": "MIT terms \n© 2025 Bob\n",
             "used_by": [{"crate": {"name": "b", "version": "2", "license": "MIT"}}]}
        ]});
        let groups = collect_groups(&data).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].packages.len(), 2);
        assert_eq!(groups[0].notices.len(), 2);
    }

    #[test]
    fn keeps_non_copyright_license_boilerplate_in_license_text() {
        let source = "MIT terms\nCopyright holders are not specified\nPermission is granted";
        let (body, notices) = split_notices(source);
        assert_eq!(body, source);
        assert!(notices.is_empty());
    }

    #[test]
    fn copyright_marker_requires_year_and_supports_comment_prefixes() {
        assert!(is_copyright_notice("Copyright (c) 2024 Example"));
        assert!(!is_copyright_notice("Copyright (c) Example"));
        assert!(is_copyright_notice("/** Copyright 2024 Example"));
        assert!(!is_copyright_notice("Copyright [year] Name"));
    }

    #[test]
    fn escapes_html_text_and_attributes_safely() {
        assert_eq!(escape_html("<tag> & 'x'", false), "&lt;tag&gt; &amp; 'x'");
        assert_eq!(
            escape_html("<tag> & 'x' \"y\"", true),
            "&lt;tag&gt; &amp; &#x27;x&#x27; &quot;y&quot;"
        );
    }
}
