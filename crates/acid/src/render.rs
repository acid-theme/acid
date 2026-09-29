//! Rendering a port's theme once per variant.
//!
//! A variant is one point of the port's matrix: a flavour, and the value of the
//! port's own axis when it declares one. The manifest says where each variant's
//! output goes; the template says what is in it.

use std::{fs, path::PathBuf};

use acid_palette::Flavor;
use anyhow::{Context, Result, bail};

use crate::{
    manifest::{Axis, Port},
    templates,
};

pub struct Variant {
    pub flavor: &'static Flavor,
    /// The value of the port's own axis, empty when it has none.
    pub format: String,
}

/// Every point of the port's matrix, flavours varying slowest.
pub fn variants(port: &Port) -> Result<Vec<Variant>> {
    let mut formats: Vec<String> = Vec::new();
    let mut has_flavor = false;

    for axis in &port.manifest.render.matrix {
        match axis {
            Axis::Flavor(name) if name == "flavor" => has_flavor = true,
            Axis::Flavor(other) => bail!("{}: unknown matrix axis {other}", port.name),
            Axis::Custom(map) => {
                for values in map.values() {
                    if !formats.is_empty() {
                        bail!("{}: only one axis of its own is supported", port.name);
                    }
                    formats = values.clone();
                }
            }
        }
    }

    if !has_flavor {
        bail!("{}: the matrix must include `flavor`", port.name);
    }
    if formats.is_empty() {
        formats.push(String::new());
    }

    Ok(acid_palette::FLAVORS
        .iter()
        .flat_map(|flavor| {
            formats.iter().map(move |format| Variant {
                flavor,
                format: format.clone(),
            })
        })
        .collect())
}

/// Where a variant's output goes, relative to the port's directory.
pub fn path(port: &Port, variant: &Variant) -> PathBuf {
    let pattern = port.manifest.render.filename.get(&variant.format);
    let filled = pattern
        .replace("{flavor}", variant.flavor.identifier)
        .replace("{flavor_name}", variant.flavor.name);
    port.dir.join(filled)
}

/// Render one variant.
pub fn one(port: &Port, variant: &Variant) -> Result<String> {
    let prefix = port
        .manifest
        .render
        .prefix
        .as_ref()
        .map_or("", |p| p.get(&variant.format));

    let rendered = templates::theme(
        &port.name,
        &templates::Theme {
            flavor: variant.flavor,
            version: acid_palette::VERSION,
            format: &variant.format,
            prefix,
        },
    )?;

    Ok(templates::normalise(rendered))
}

/// Render every variant of every port. Returns the paths written.
pub fn all(ports: &[Port], write: bool) -> Result<Vec<PathBuf>> {
    let mut stale = Vec::new();

    for port in ports {
        for variant in variants(port)? {
            let rendered = one(port, &variant)?;
            let destination = path(port, &variant);

            let current = fs::read_to_string(&destination).ok();
            if current.as_deref() == Some(rendered.as_str()) {
                continue;
            }

            if write {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("cannot create {}", parent.display()))?;
                }
                fs::write(&destination, &rendered)
                    .with_context(|| format!("cannot write {}", destination.display()))?;
            }
            stale.push(destination);
        }
    }

    Ok(stale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Manifest, PerVariant, Render};
    use std::collections::BTreeMap;

    fn port(matrix: Vec<Axis>, filename: PerVariant) -> Port {
        Port {
            name: "example".to_owned(),
            dir: PathBuf::from("ports/example"),
            manifest: Manifest {
                title: "Example".to_owned(),
                description: "An example.".to_owned(),
                repo: None,
                test: None,
                preview: None,
                render: Render {
                    matrix,
                    filename,
                    prefix: None,
                },
            },
        }
    }

    fn custom(key: &str, values: &[&str]) -> Axis {
        let mut map = BTreeMap::new();
        map.insert(
            key.to_owned(),
            values.iter().map(|v| (*v).to_owned()).collect(),
        );
        Axis::Custom(map)
    }

    #[test]
    fn a_flavour_axis_renders_once_per_flavour() {
        let port = port(
            vec![Axis::Flavor("flavor".to_owned())],
            PerVariant::Same("dist/acid-{flavor}.conf".to_owned()),
        );
        let variants = variants(&port).unwrap();
        assert_eq!(variants.len(), acid_palette::FLAVORS.len());
        assert!(variants.iter().all(|v| v.format.is_empty()));
    }

    /// Flavours vary slowest, so a port's own axis groups within each flavour.
    #[test]
    fn a_second_axis_multiplies_with_the_flavours() {
        let port = port(
            vec![
                Axis::Flavor("flavor".to_owned()),
                custom("format", &["theme", "conf"]),
            ],
            PerVariant::Same("dist/{flavor}".to_owned()),
        );
        let variants = variants(&port).unwrap();
        assert_eq!(variants.len(), acid_palette::FLAVORS.len() * 2);
        assert_eq!(variants[0].flavor.identifier, variants[1].flavor.identifier);
        assert_eq!(variants[0].format, "theme");
        assert_eq!(variants[1].format, "conf");
    }

    #[test]
    fn a_matrix_without_flavour_is_refused() {
        let port = port(
            vec![custom("format", &["theme"])],
            PerVariant::Same("dist/x".to_owned()),
        );
        assert!(variants(&port).is_err());
    }

    #[test]
    fn filenames_substitute_the_flavour() {
        let port = port(
            vec![Axis::Flavor("flavor".to_owned())],
            PerVariant::Same("dist/acid-{flavor}.conf".to_owned()),
        );
        let variant = &variants(&port).unwrap()[0];
        assert!(path(&port, variant).ends_with("dist/acid-acetic.conf"));
    }

    /// The name form is what fish's theme files are called.
    #[test]
    fn filenames_can_use_the_display_name() {
        let port = port(
            vec![Axis::Flavor("flavor".to_owned())],
            PerVariant::Same("dist/Acid {flavor_name}.theme".to_owned()),
        );
        let variant = &variants(&port).unwrap()[0];
        assert!(path(&port, variant).ends_with("dist/Acid Acetic.theme"));
    }

    #[test]
    fn a_filename_can_differ_per_variant() {
        let mut map = BTreeMap::new();
        map.insert("theme".to_owned(), "dist/themes/{flavor}".to_owned());
        map.insert("conf".to_owned(), "dist/conf.d/{flavor}".to_owned());
        let port = port(
            vec![
                Axis::Flavor("flavor".to_owned()),
                custom("format", &["theme", "conf"]),
            ],
            PerVariant::PerVariant(map),
        );
        let variants = variants(&port).unwrap();
        assert!(path(&port, &variants[0]).ends_with("dist/themes/acetic"));
        assert!(path(&port, &variants[1]).ends_with("dist/conf.d/acetic"));
    }

    /// A variant the map does not mention would otherwise render to nowhere.
    #[test]
    fn a_missing_variant_yields_an_empty_name() {
        let mut map = BTreeMap::new();
        map.insert("theme".to_owned(), "dist/themes/{flavor}".to_owned());
        let port = port(
            vec![
                Axis::Flavor("flavor".to_owned()),
                custom("format", &["theme", "conf"]),
            ],
            PerVariant::PerVariant(map),
        );
        let variants = variants(&port).unwrap();
        let conf = variants.iter().find(|v| v.format == "conf").unwrap();
        assert_eq!(path(&port, conf), port.dir);
    }
}
