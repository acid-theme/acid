//! `acidify` renders Acid ports from Tera templates.
//!
//! One template per port; one output per matrix combination. This follows the
//! shape of Catppuccin's `whiskers`, which cannot be reused directly because it
//! renders from the Catppuccin palette compiled into it.

mod filters;
mod template;

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use acid_palette::json;
use anyhow::{Context as _, Result, bail};
use clap::Parser;
use serde_json::{Map, Value};
use tera::{Context, Tera};

use crate::template::Template;

#[derive(Parser)]
#[command(
    version,
    about = "Render Acid ports from Tera templates.",
    after_help = "Each template declares its own matrix and output filename in \
                  its `acidify:` frontmatter block."
)]
struct Args {
    /// Templates to render.
    #[arg(required = true, value_name = "TEMPLATE")]
    templates: Vec<PathBuf>,

    /// Where rendered filenames resolve. Defaults to each template's directory.
    #[arg(short, long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Write nothing; fail if any output is missing or stale. For CI.
    #[arg(long)]
    check: bool,

    /// Write to stdout instead of to files.
    #[arg(long, conflicts_with = "check")]
    stdout: bool,

    /// Print the context of each combination instead of rendering. For authoring.
    #[arg(long, conflicts_with_all = ["check", "stdout"])]
    context: bool,
}

/// One rendered output, before it touches the disk.
struct Rendered {
    path: Option<PathBuf>,
    contents: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("acidify: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Returns whether every `--check` comparison passed.
fn run(args: &Args) -> Result<bool> {
    let mut up_to_date = true;

    for path in &args.templates {
        let template = Template::read(path)?;
        let output_dir = args.output_dir.clone().unwrap_or_else(|| {
            path.parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .to_path_buf()
        });

        for combination in template.combinations() {
            let context = build_context(&template, &combination)?;

            if args.context {
                println!(
                    "# {} {}\n{}",
                    template.source,
                    describe(&combination),
                    serde_json::to_string_pretty(&context.clone().into_json())?
                );
                continue;
            }

            let rendered = render(&template, &context, &output_dir)?;

            match (&rendered.path, args.stdout, args.check) {
                (_, true, _) | (None, _, _) => print!("{}", rendered.contents),
                (Some(path), _, true) => {
                    if fs::read_to_string(path).ok().as_deref() != Some(&rendered.contents) {
                        eprintln!("acidify: {} is stale", path.display());
                        up_to_date = false;
                    }
                }
                (Some(path), _, false) => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("cannot create {}", parent.display()))?;
                    }
                    fs::write(path, &rendered.contents)
                        .with_context(|| format!("cannot write {}", path.display()))?;
                    println!("acidify: wrote {}", path.display());
                }
            }
        }
    }

    Ok(up_to_date)
}

/// Build the variables a single combination renders with.
///
/// Precedence, lowest first: the palette, frontmatter variables, then the
/// matrix bindings — so a matrix axis always wins over a frontmatter default.
fn build_context(template: &Template, combination: &Map<String, Value>) -> Result<Context> {
    let palette = template_palette();
    let mut variables = template.variables.clone();

    for (key, value) in combination {
        variables.insert(key.clone(), value.clone());
    }

    // Resolve the `flavor` axis from an identifier to the whole flavour.
    if let Some(Value::String(identifier)) = variables.get("flavor").cloned() {
        let flavor = palette
            .get(&identifier)
            .with_context(|| format!("{}: no such flavour `{identifier}`", template.source))?;
        variables.insert("flavor".to_owned(), flavor.clone());

        // With a flavour in hand, `accent` can become that flavour's colour.
        if let Some(Value::String(accent)) = variables.get("accent").cloned() {
            let color = flavor
                .get("accents")
                .and_then(Value::as_array)
                .and_then(|accents| {
                    accents
                        .iter()
                        .find(|color| color["identifier"] == accent.as_str())
                })
                .with_context(|| {
                    format!(
                        "{}: `{identifier}` has no accent `{accent}`",
                        template.source
                    )
                })?;
            variables.insert("accent".to_owned(), color.clone());
        }
    }

    let mut context = Context::new();
    context.insert("flavors", &palette);
    context.insert("version", &acid_palette::VERSION);
    for (key, value) in variables {
        context.insert(key, &value);
    }
    Ok(context)
}

/// The palette as templates see it.
///
/// Colours flatten to their hex string, so `{{ flavor.colors.base }}` prints
/// `#000000` and the derived forms come from filters. `flavor.color_list` and
/// `flavor.accents` keep the full objects, in canonical order, for templates
/// that need a colour's name, order or accent flag.
fn template_palette() -> Value {
    let mut palette = json::palette();
    let flavors = palette["flavors"]
        .as_object_mut()
        .expect("the palette holds an object keyed by flavour");

    for flavor in flavors.values_mut() {
        let colors = flavor["colors"]
            .as_object()
            .expect("a flavour's colours are an object")
            .clone();

        flavor["color_list"] = Value::Array(colors.values().cloned().collect());
        flavor["colors"] = Value::Object(
            colors
                .into_iter()
                .map(|(identifier, color)| (identifier, color["hex"].clone()))
                .collect(),
        );
    }

    palette["flavors"].take()
}

fn render(template: &Template, context: &Context, output_dir: &Path) -> Result<Rendered> {
    let mut tera = Tera::default();
    filters::register(&mut tera);

    let contents = tera
        .render_str(&template.body, context)
        .with_context(|| format!("{}: template failed to render", template.source))?;

    let path = match &template.filename {
        None => None,
        Some(filename) => {
            let rendered = tera
                .render_str(filename, context)
                .with_context(|| format!("{}: filename failed to render", template.source))?;
            let rendered = rendered.trim();
            if rendered.is_empty() {
                bail!("{}: filename rendered empty", template.source);
            }
            Some(output_dir.join(rendered))
        }
    };

    Ok(Rendered { path, contents })
}

/// A one-line summary of a combination, for `--context` headers.
fn describe(combination: &Map<String, Value>) -> String {
    combination
        .iter()
        .map(|(key, value)| match value {
            Value::String(s) => format!("{key}={s}"),
            other => format!("{key}={other}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(source: &str) -> Vec<(Option<PathBuf>, String)> {
        let template = Template::parse(source, "test").unwrap();
        template
            .combinations()
            .iter()
            .map(|combination| {
                let context = build_context(&template, combination).unwrap();
                let rendered = render(&template, &context, Path::new("out")).unwrap();
                (rendered.path, rendered.contents)
            })
            .collect()
    }

    #[test]
    fn renders_one_output_per_flavour() {
        let outputs = render_to_string(
            "---\nacidify:\n  matrix: [flavor]\n  filename: \"acid-{{ flavor.identifier }}.toml\"\n---\nbg = \"{{ flavor.colors.base }}\"\n",
        );
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].0, Some(PathBuf::from("out/acid-acetic.toml")));
        assert_eq!(outputs[0].1, "bg = \"#000000\"\n");
        assert_eq!(outputs[1].0, Some(PathBuf::from("out/acid-citric.toml")));
        assert_eq!(outputs[1].1, "bg = \"#1c1b19\"\n");
    }

    /// A colour object renders as its hex string, so `.hex` is optional.
    #[test]
    fn colours_stringify_to_hex() {
        let outputs =
            render_to_string("---\nacidify:\n  matrix: [flavor]\n---\n{{ flavor.colors.red }}");
        assert_eq!(outputs[0].1, "#ff4536");
        assert_eq!(outputs[1].1, "#de5a48");
    }

    #[test]
    fn every_flavour_is_reachable_without_a_matrix() {
        let outputs =
            render_to_string("---\nacidify: {}\n---\n{{ flavors.citric.colors.green | bare }}");
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].0, None);
        assert_eq!(outputs[0].1, "a3ab2f");
    }

    /// Every template can stamp the version that produced its output.
    #[test]
    fn version_is_always_available() {
        let outputs = render_to_string("---\nacidify: {}\n---\n{{ version }}");
        assert_eq!(outputs[0].1, acid_palette::VERSION);
        assert!(!acid_palette::VERSION.is_empty());
    }

    #[test]
    fn matrix_bindings_override_frontmatter_defaults() {
        let outputs = render_to_string(
            "---\nacidify:\n  matrix:\n    - transparent: [true]\ntransparent: false\n---\n{{ transparent }}",
        );
        assert_eq!(outputs[0].1, "true");
    }

    #[test]
    fn accent_resolves_against_the_current_flavour() {
        let outputs = render_to_string(
            "---\nacidify:\n  matrix: [flavor, accent]\n  filename: \"{{ flavor.identifier }}-{{ accent.identifier }}.txt\"\n---\n{{ accent.hex }}",
        );
        assert_eq!(outputs.len(), 14);
        assert_eq!(outputs[0].0, Some(PathBuf::from("out/acetic-red.txt")));
        assert_eq!(outputs[0].1, "#ff4536");
        assert_eq!(outputs[7].0, Some(PathBuf::from("out/citric-red.txt")));
        assert_eq!(outputs[7].1, "#de5a48");
    }

    /// Metadata stays reachable for templates that iterate the palette.
    #[test]
    fn color_list_keeps_names_and_order() {
        let outputs = render_to_string(
            "---\nacidify:\n  matrix: [flavor]\n---\n{% for c in flavor.color_list %}{{ c.order }}:{{ c.identifier }}:{{ c.name }}:{{ c.accent }} {% endfor %}",
        );
        assert!(
            outputs[0]
                .1
                .starts_with("0:red:Red:true 1:orange:Orange:true ")
        );
        assert!(outputs[0].1.trim_end().ends_with("18:crust:Crust:false"));
    }

    #[test]
    fn ports_can_branch_on_inverted_depth() {
        let outputs = render_to_string(
            "---\nacidify:\n  matrix: [flavor]\n---\n{% if flavor.invertedDepth %}up{% else %}down{% endif %}",
        );
        assert_eq!(outputs[0].1, "up");
        assert_eq!(outputs[1].1, "down");
    }
}
