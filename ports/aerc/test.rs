// aerc refuses to start when a styleset names an object its parser does not
// know, and its manual and bundled stylesets list objects the binary rejects.
// So the binary is asked, one object at a time, rather than the documentation.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["aerc"]) {
        return Ok(());
    }

    let root = std::env::temp_dir().join("aerc-config");
    std::fs::create_dir_all(root.join("aerc/stylesets"))?;
    std::fs::write(root.join("aerc/aerc.conf"), "[ui]\nstyleset-name=probe\n")?;
    // Left deliberately lax: aerc checks the styleset first, then complains
    // about these permissions, which makes every probe exit immediately.
    std::fs::write(
        root.join("aerc/accounts.conf"),
        "[default]\nsource=maildir:///tmp/mail\nfrom=a <a@b.c>\n",
    )?;

    // Returns the object aerc rejected, if it rejected one.
    let probe = |contents: &str| -> Option<String> {
        std::fs::write(root.join("aerc/stylesets/probe"), contents).ok()?;
        let output = exec(
            "env",
            &[
                &format!("XDG_CONFIG_HOME={}", root.display()),
                "timeout", "10", "aerc",
            ],
        );
        output
            .lines()
            .find_map(|line| line.split("unknown style object: ").nth(1))
            .map(|name| name.trim().to_owned())
    };

    let manual = "/usr/share/man/man7/aerc-stylesets.7.gz";
    let documented: Vec<String> = exec("zcat", &["-f", manual])
        .split("\n.SH STYLE OBJECTS")
        .nth(1)
        .unwrap_or_default()
        .split("\n.SH ")
        .next()
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.strip_prefix("\\fB")?.strip_suffix("\\fR"))
        .filter(|name| name.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()))
        .map(str::to_owned)
        .collect();

    if documented.len() < 30 {
        h.fail(format!("could not read aerc's manual ({} objects)", documented.len()));
        return Ok(());
    }

    // Which documented objects this build of aerc actually accepts.
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for object in &documented {
        if probe(&format!("{object}.fg=#ff0000\n")).is_some() {
            rejected.push(object.clone());
        } else {
            accepted.push(object.clone());
        }
    }
    h.note(format!(
        "aerc documents {} objects and accepts {}",
        documented.len(),
        accepted.len()
    ));
    if !rejected.is_empty() {
        h.note(format!("it rejects: {}", rejected.join(" ")));
    }

    for flavour in h.flavours() {
        let styleset = read(&h.dist(&format!("acid-{}", flavour.identifier)));

        // The check that matters: aerc starts with this styleset.
        match probe(&styleset) {
            Some(object) => h.fail(format!(
                "{}: aerc rejects the styleset at {object}",
                flavour.identifier
            )),
            None => h.pass(format!("{}: aerc accepts the styleset", flavour.identifier)),
        }

        let used: Vec<&str> = styleset
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| line.split(['.', '=']).next())
            .filter(|name| *name != "*")
            .collect();

        let unaccepted: Vec<&&str> = used
            .iter()
            .filter(|name| !accepted.iter().any(|a| a == **name))
            .collect();
        h.check(
            unaccepted.is_empty(),
            format!(
                "{}: every object used is one aerc accepts{}",
                flavour.identifier,
                if unaccepted.is_empty() { String::new() } else { format!("; {unaccepted:?}") }
            ),
        );

        let unstyled: Vec<&String> = accepted
            .iter()
            .filter(|object| !used.contains(&object.as_str()))
            .collect();
        h.check(
            unstyled.is_empty(),
            format!(
                "{}: every object aerc accepts is styled{}",
                flavour.identifier,
                if unstyled.is_empty() { String::new() } else { format!("; {unstyled:?}") }
            ),
        );
    }

    h.check(
        probe("nonsense_object.fg=#ff0000\n").as_deref() == Some("nonsense_object"),
        "control: aerc rejects an invented object",
    );

    Ok(())
}
