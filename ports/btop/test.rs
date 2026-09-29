// btop accepts anything: a malformed colour, an unknown key and a missing themes
// directory all pass without a word. So the theme is checked against the key set
// btop itself ships, rather than by asking btop.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["btop"]) {
        return Ok(());
    }

    let mut known: Vec<String> = Vec::new();
    for entry in std::fs::read_dir("/usr/share/btop/themes")? {
        for line in read(&entry?.path()).lines() {
            if let Some(key) = line.strip_prefix("theme[").and_then(|r| r.split(']').next())
                && !known.iter().any(|k| k == key)
            {
                known.push(key.to_owned());
            }
        }
    }
    known.sort();

    if known.len() < 20 {
        h.fail(format!("could not read btop's key set (found {})", known.len()));
        return Ok(());
    }
    h.note(format!("btop ships {} theme keys", known.len()));

    for flavour in h.flavours() {
        let theme = read(&h.dist(&format!("acid-{}.theme", flavour.identifier)));
        let entries: Vec<(String, String)> = theme
            .lines()
            .filter_map(|line| line.strip_prefix("theme["))
            .filter_map(|rest| rest.split_once("]="))
            .map(|(key, value)| (key.to_owned(), value.trim_matches('"').to_owned()))
            .collect();

        let unknown: Vec<&String> = entries
            .iter()
            .map(|(key, _)| key)
            .filter(|key| !known.contains(key))
            .collect();
        h.check(
            unknown.is_empty(),
            format!("{}: all {} keys are ones btop uses", flavour.identifier, entries.len()),
        );

        let malformed: Vec<&String> = entries
            .iter()
            .filter(|(_, value)| {
                !(value.len() == 7
                    && value.starts_with('#')
                    && value[1..].chars().all(|c| c.is_ascii_hexdigit()))
            })
            .map(|(key, _)| key)
            .collect();
        h.check(
            malformed.is_empty(),
            format!("{}: all {} values are #rrggbb", flavour.identifier, entries.len()),
        );

        let missing: Vec<&String> = known
            .iter()
            .filter(|key| !entries.iter().any(|(k, _)| k == *key))
            .collect();
        h.check(
            missing.is_empty(),
            format!(
                "{}: every key btop ships is set{}",
                flavour.identifier,
                if missing.is_empty() { String::new() } else { format!("; missing {missing:?}") }
            ),
        );
    }

    h.check(
        !known.iter().any(|k| k == "nonsense_key"),
        "control: an invented key is absent from btop's set",
    );

    Ok(())
}
