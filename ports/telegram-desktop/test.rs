// Telegram is a GUI client that cannot load a theme headlessly, so this checks
// the palette's structure against Telegram's own default, vendored beside the
// template. It cannot tell whether the result looks good.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    fn entries(text: &str) -> Vec<(String, String)> {
        text.lines()
            .filter_map(|line| {
                let (name, rest) = line.split_once(':')?;
                let name = name.trim();
                if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return None;
                }
                let value = rest.split(';').next()?.trim();
                Some((name.to_owned(), value.to_owned()))
            })
            .collect()
    }

    let upstream = entries(&read(
        &h.root.join("ports/telegram-desktop/upstream.palette"),
    ));
    h.check(
        upstream.len() > 500,
        format!("Telegram's default palette has {} keys", upstream.len()),
    );

    let is_colour = |value: &str| {
        let body = value.trim_start_matches('#');
        value.starts_with('#')
            && matches!(body.len(), 3 | 4 | 6 | 8)
            && body.chars().all(|c| c.is_ascii_hexdigit())
    };
    let is_key = |value: &str| {
        !value.is_empty()
            && value.chars().all(|c| c.is_ascii_alphanumeric())
            && value.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
    };

    for flavour in h.flavours() {
        let theme = entries(&read(&h.dist(&format!(
            "acid-{}.tdesktop-palette",
            flavour.identifier
        ))));
        let names: Vec<&String> = theme.iter().map(|(name, _)| name).collect();

        let unknown: Vec<&&String> = names
            .iter()
            .filter(|name| !upstream.iter().any(|(u, _)| u == **name))
            .collect();
        h.check(
            unknown.is_empty(),
            format!(
                "{}: every key exists upstream — {} keys",
                flavour.identifier,
                theme.len()
            ),
        );

        let missing: Vec<&String> = upstream
            .iter()
            .map(|(name, _)| name)
            .filter(|name| !names.contains(name))
            .collect();
        h.check(
            missing.is_empty(),
            format!("{}: no key is left out", flavour.identifier),
        );

        // A value is a literal colour or another key's name, and nothing else.
        // Telegram's source palette also allows "value | fallback", but its
        // theme parser refuses it.
        let bad: Vec<&(String, String)> = theme
            .iter()
            .filter(|(_, value)| !is_colour(value) && !is_key(value))
            .collect();
        h.check(
            bad.is_empty(),
            format!(
                "{}: every value is a colour or a key{}",
                flavour.identifier,
                if bad.is_empty() { String::new() } else { format!("; {:?}", &bad[..1]) }
            ),
        );

        let dangling: Vec<&(String, String)> = theme
            .iter()
            .filter(|(_, value)| is_key(value) && !names.contains(&value))
            .collect();
        h.check(
            dangling.is_empty(),
            format!("{}: every reference resolves", flavour.identifier),
        );

        // A cycle would leave a colour undefined.
        let cycles: Vec<&String> = names
            .iter()
            .copied()
            .filter(|name| {
                let mut seen: Vec<&str> = Vec::new();
                let mut current: &str = name;
                loop {
                    let Some((_, value)) = theme.iter().find(|(n, _)| n == current) else {
                        return false;
                    };
                    if !is_key(value) {
                        return false;
                    }
                    if seen.contains(&value.as_str()) {
                        return true;
                    }
                    seen.push(value);
                    current = value;
                }
            })
            .collect();
        h.check(
            cycles.is_empty(),
            format!("{}: no reference cycles", flavour.identifier),
        );
    }

    h.check(!is_colour("#00zz00"), "control: a malformed colour is refused");
    h.check(
        !is_colour("#2dad2d | boxTextFgGood") && !is_key("#2dad2d | boxTextFgGood"),
        "control: the source-only fallback syntax is refused",
    );

    Ok(())
}
