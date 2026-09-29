// swaylock exits rather than locking when it meets a key it does not know, so
// every key is checked against its own --help output.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["swaylock"]) {
        return Ok(());
    }

    let help = exec("swaylock", &["--help"]);
    let known: Vec<&str> = help
        .lines()
        .filter_map(|line| line.split("--").nth(1))
        .map(|rest| rest.split([' ', '\t', '<']).next().unwrap_or_default())
        .filter(|name| !name.is_empty())
        .collect();

    if known.len() < 20 {
        h.fail(format!("could not read swaylock's option list (found {})", known.len()));
        return Ok(());
    }
    h.note(format!("swaylock advertises {} options", known.len()));

    let colour_options: Vec<&&str> = known
        .iter()
        .filter(|name| **name == "color" || name.ends_with("-color"))
        .collect();

    for flavour in h.flavours() {
        let theme = read(&h.dist(&format!("acid-{}.conf", flavour.identifier)));
        let entries: Vec<(&str, &str)> = theme
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.split_once('='))
            .collect();

        let unknown: Vec<&str> = entries
            .iter()
            .map(|(key, _)| *key)
            .filter(|key| !known.contains(key))
            .collect();
        h.check(
            unknown.is_empty(),
            format!(
                "{}: all {} keys are known to swaylock{}",
                flavour.identifier,
                entries.len(),
                if unknown.is_empty() { String::new() } else { format!("; unknown {unknown:?}") }
            ),
        );

        // swaylock documents colours as <rrggbb[aa]>: bare hex, no leading #.
        let malformed: Vec<&(&str, &str)> = entries
            .iter()
            .filter(|(_, value)| {
                !(matches!(value.len(), 6 | 8) && value.chars().all(|c| c.is_ascii_hexdigit()))
            })
            .collect();
        h.check(
            malformed.is_empty(),
            format!(
                "{}: all {} values are bare 6 or 8 digit hex",
                flavour.identifier,
                entries.len()
            ),
        );

        let missing: Vec<&&&str> = colour_options
            .iter()
            .filter(|option| !entries.iter().any(|(key, _)| *key == ***option))
            .collect();
        h.check(
            missing.is_empty(),
            format!(
                "{}: every colour option swaylock offers is set{}",
                flavour.identifier,
                if missing.is_empty() { String::new() } else { format!("; missing {missing:?}") }
            ),
        );
    }

    h.check(
        !known.contains(&"nonsense-color"),
        "control: an invented key is absent from the option list",
    );

    Ok(())
}
