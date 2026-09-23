//! Template parsing: YAML frontmatter, and the matrix that turns one template
//! into many outputs.
//!
//! A template looks like this:
//!
//! ```text
//! ---
//! acidify:
//!   version: "0.1"
//!   matrix:
//!     - flavor
//!   filename: "themes/acid-{{ flavor.identifier }}.toml"
//! transparent: false          # any other key is passed through as a variable
//! ---
//! background = "{{ flavor.colors.base }}"
//! ```

use std::{fs, path::Path};

use anyhow::{Context as _, Result, bail};
use serde::Deserialize;
use serde_json::{Map, Value};

const DELIMITER: &str = "---";

/// One axis of the render matrix.
#[derive(Debug, Clone, PartialEq)]
pub enum Dimension {
    /// Render once per flavour, binding `flavor`.
    Flavor,
    /// Render once per accent, binding `accent`.
    Accent,
    /// Render once per value, binding the given name.
    Custom { key: String, values: Vec<Value> },
}

impl Dimension {
    fn key(&self) -> &str {
        match self {
            Self::Flavor => "flavor",
            Self::Accent => "accent",
            Self::Custom { key, .. } => key,
        }
    }

    /// The values this axis binds. Flavours and accents bind identifiers here
    /// and are resolved to full colour objects at render time, once the
    /// combination is known.
    fn values(&self) -> Vec<Value> {
        match self {
            Self::Flavor => acid_palette::FLAVORS
                .iter()
                .map(|flavor| Value::String(flavor.identifier.to_owned()))
                .collect(),
            Self::Accent => acid_palette::ACETIC
                .accents()
                .map(|color| Value::String(color.identifier.to_owned()))
                .collect(),
            Self::Custom { values, .. } => values.clone(),
        }
    }
}

/// The `acidify:` block of the frontmatter.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMeta {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    matrix: Vec<serde_yaml_ng::Value>,
    #[serde(default)]
    filename: Option<String>,
}

#[derive(Debug)]
pub struct Template {
    /// Where the template came from, for error messages.
    pub source: String,
    /// Declared matrix axes, outermost first.
    pub matrix: Vec<Dimension>,
    /// Tera template for the output path, relative to the output directory.
    pub filename: Option<String>,
    /// Frontmatter keys other than `acidify:`, available as variables.
    pub variables: Map<String, Value>,
    /// Everything after the frontmatter.
    pub body: String,
}

impl Template {
    pub fn read(path: &Path) -> Result<Self> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
        Self::parse(&contents, &path.display().to_string())
    }

    pub fn parse(contents: &str, source: &str) -> Result<Self> {
        let (frontmatter, body) = split(contents)
            .with_context(|| format!("{source}: frontmatter is not closed by `---`"))?;

        let mut variables: Map<String, Value> = if frontmatter.trim().is_empty() {
            Map::new()
        } else {
            let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(frontmatter)
                .with_context(|| format!("{source}: frontmatter is not valid YAML"))?;
            match serde_json::to_value(parsed)? {
                Value::Object(map) => map,
                Value::Null => Map::new(),
                _ => bail!("{source}: frontmatter must be a mapping"),
            }
        };

        let raw = match variables.remove("acidify") {
            Some(value) => serde_json::from_value::<RawMeta>(value)
                .with_context(|| format!("{source}: the `acidify` block is malformed"))?,
            None => bail!("{source}: frontmatter has no `acidify` block"),
        };

        if let Some(version) = &raw.version {
            let expected = env!("CARGO_PKG_VERSION");
            if !expected.starts_with(version.trim_end_matches('.')) && version != expected {
                eprintln!(
                    "acidify: {source} asks for version {version}, this is {expected}; \
                     rendering anyway"
                );
            }
        }

        let matrix = raw
            .matrix
            .iter()
            .map(|entry| dimension(entry, source))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            source: source.to_owned(),
            matrix,
            filename: raw.filename,
            variables,
            body: body.to_owned(),
        })
    }

    /// Every combination of the matrix axes, outermost axis varying slowest.
    /// A template with no matrix renders exactly once, with no extra bindings.
    pub fn combinations(&self) -> Vec<Map<String, Value>> {
        let mut combinations = vec![Map::new()];
        for dimension in &self.matrix {
            let key = dimension.key();
            combinations = combinations
                .into_iter()
                .flat_map(|combination| {
                    dimension.values().into_iter().map(move |value| {
                        let mut next = combination.clone();
                        next.insert(key.to_owned(), value);
                        next
                    })
                })
                .collect();
        }
        combinations
    }
}

/// Split leading `---`-delimited frontmatter from the body.
fn split(contents: &str) -> Option<(&str, &str)> {
    let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
    let rest = contents
        .strip_prefix(DELIMITER)?
        .strip_prefix('\n')
        .or_else(|| contents.strip_prefix(DELIMITER)?.strip_prefix("\r\n"))?;

    // The closing delimiter is a line of its own.
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == DELIMITER {
            let body = &rest[offset + line.len()..];
            return Some((&rest[..offset], body));
        }
        offset += line.len();
    }
    None
}

fn dimension(entry: &serde_yaml_ng::Value, source: &str) -> Result<Dimension> {
    match entry {
        serde_yaml_ng::Value::String(name) => match name.as_str() {
            "flavor" | "flavour" => Ok(Dimension::Flavor),
            "accent" => Ok(Dimension::Accent),
            other => bail!(
                "{source}: `{other}` is not a known matrix axis; \
                 use `flavor`, `accent`, or `{{ {other}: [..] }}`"
            ),
        },
        serde_yaml_ng::Value::Mapping(mapping) if mapping.len() == 1 => {
            let (key, values) = mapping.iter().next().expect("length was checked");
            let key = key
                .as_str()
                .with_context(|| format!("{source}: matrix keys must be strings"))?
                .to_owned();
            let values = match serde_json::to_value(values.clone())? {
                Value::Array(values) if !values.is_empty() => values,
                Value::Array(_) => bail!("{source}: matrix axis `{key}` has no values"),
                _ => bail!("{source}: matrix axis `{key}` must be a list"),
            };
            Ok(Dimension::Custom { key, values })
        }
        _ => bail!("{source}: each matrix entry must be a name or a single-key mapping"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE: &str = "---\nacidify:\n  matrix:\n    - flavor\n  filename: \"acid-{{ flavor.identifier }}.toml\"\ntransparent: false\n---\nbase = \"{{ flavor.colors.base }}\"\n";

    #[test]
    fn splits_frontmatter_from_body() {
        let template = Template::parse(SIMPLE, "test").unwrap();
        assert_eq!(template.matrix, vec![Dimension::Flavor]);
        assert_eq!(
            template.filename.as_deref(),
            Some("acid-{{ flavor.identifier }}.toml")
        );
        assert_eq!(template.variables["transparent"], false);
        assert_eq!(template.body, "base = \"{{ flavor.colors.base }}\"\n");
    }

    #[test]
    fn body_keeps_its_own_delimiters() {
        let template = Template::parse("---\nacidify: {}\n---\na\n---\nb\n", "test").unwrap();
        assert_eq!(template.body, "a\n---\nb\n");
        assert!(template.matrix.is_empty());
    }

    #[test]
    fn no_matrix_renders_once() {
        let template = Template::parse("---\nacidify: {}\n---\nbody", "test").unwrap();
        assert_eq!(template.combinations(), vec![Map::new()]);
    }

    #[test]
    fn matrix_is_a_cartesian_product() {
        let source =
            "---\nacidify:\n  matrix:\n    - flavor\n    - transparent: [true, false]\n---\n";
        let combinations = Template::parse(source, "test").unwrap().combinations();
        assert_eq!(combinations.len(), 4);
        // The outermost axis varies slowest.
        assert_eq!(combinations[0]["flavor"], "acetic");
        assert_eq!(combinations[0]["transparent"], true);
        assert_eq!(combinations[1]["transparent"], false);
        assert_eq!(combinations[2]["flavor"], "citric");
    }

    #[test]
    fn accent_axis_covers_every_accent() {
        let source = "---\nacidify:\n  matrix: [flavor, accent]\n---\n";
        let combinations = Template::parse(source, "test").unwrap().combinations();
        assert_eq!(combinations.len(), 14);
    }

    #[test]
    fn rejects_bad_frontmatter() {
        for source in [
            "no frontmatter at all\n",
            "---\nacidify:\n  matrix: [nonsense]\n---\n",
            "---\nnothing: here\n---\n",
            "---\nacidify:\n  unknown_key: 1\n---\n",
            "---\nacidify: {}\nunclosed: true\n",
        ] {
            assert!(
                Template::parse(source, "test").is_err(),
                "accepted: {source:?}"
            );
        }
    }
}
