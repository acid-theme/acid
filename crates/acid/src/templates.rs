//! The compiled templates.
//!
//! Every port contributes two: its theme and its README. Both are compiled into
//! the binary, so a template naming a colour role that does not exist is a build
//! error rather than a surprise at render time.

use acid_palette::Flavor;
use anyhow::{Result, bail};
use askama::Template;

use crate::filters;

/// What a theme template can refer to.
pub struct Theme<'a> {
    pub flavor: &'a Flavor,
    pub version: &'a str,
    /// The value of the port's own matrix axis, empty when it has none.
    pub format: &'a str,
    /// A string the manifest supplies for this variant, empty by default.
    pub prefix: &'a str,
}

/// What a README template can refer to.
pub struct Readme<'a> {
    pub hub: &'a str,
    /// The preview table, empty when the port has no preview.
    pub preview: &'a str,
}

/// The palette reference in docs/.
#[derive(Template)]
#[template(path = "palette.md.tera", escape = "none")]
pub struct Palette<'a> {
    pub version: &'a str,
    pub flavors: &'a [Flavor],
}

macro_rules! acid_templates {
    ($($ident:ident => $name:literal, $theme:literal, $readme:literal;)*) => {
        mod compiled {
            use acid_palette::Flavor;
            use askama::Template;
            use crate::filters;

            $(
                #[derive(Template)]
                #[template(path = $theme, escape = "none")]
                #[allow(dead_code, reason = "read by the generated render code")]
                pub struct $ident<'a> {
                    pub flavor: &'a Flavor,
                    pub version: &'a str,
                    pub format: &'a str,
                    pub prefix: &'a str,
                }
            )*
        }

        mod readmes {
            use askama::Template;

            $(
                #[derive(Template)]
                #[template(path = $readme, escape = "none")]
                #[allow(dead_code, reason = "read by the generated render code")]
                pub struct $ident<'a> {
                    pub hub: &'a str,
                    pub preview: &'a str,
                }
            )*
        }

        pub fn theme(port: &str, data: &Theme<'_>) -> Result<String> {
            match port {
                $($name => Ok(compiled::$ident {
                    flavor: data.flavor,
                    version: data.version,
                    format: data.format,
                    prefix: data.prefix,
                }
                .render()?),)*
                other => bail!("no theme template compiled for {other}"),
            }
        }

        pub fn readme(port: &str, data: &Readme<'_>) -> Result<String> {
            match port {
                $($name => Ok(readmes::$ident {
                    hub: data.hub,
                    preview: data.preview,
                }
                .render()?),)*
                other => bail!("no README template compiled for {other}"),
            }
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/ports.rs"));

/// A generated file ends with exactly one newline, whatever the template did.
pub fn normalise(mut rendered: String) -> String {
    while rendered.ends_with('\n') {
        rendered.pop();
    }
    rendered.push('\n');
    rendered
}

#[cfg(test)]
mod tests {
    use super::normalise;

    #[test]
    fn every_generated_file_ends_with_one_newline() {
        assert_eq!(normalise("a".to_owned()), "a\n");
        assert_eq!(normalise("a\n".to_owned()), "a\n");
        assert_eq!(normalise("a\n\n\n".to_owned()), "a\n");
        assert_eq!(normalise(String::new()), "\n");
    }
}
