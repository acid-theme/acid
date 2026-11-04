// tmux checks an option when it is set: an unknown name or a malformed style is
// an error from source-file. So each theme is sourced into a real server, every
// option is read back, and the set is compared with the colour options this
// tmux has.

// Colour options the theme leaves alone on purpose. Each must still be one tmux
// knows, so the list cannot go stale unnoticed.
const EXEMPT: &[(&str, &str)] = &[
    ("window-style", "panes keep the terminal's own background"),
    ("window-active-style", "panes keep the terminal's own background"),
    ("cursor-colour", "the cursor is the terminal's"),
    ("cursor-style", "a cursor shape, not a colour"),
    ("prompt-cursor-colour", "the cursor is the terminal's"),
    ("prompt-cursor-style", "a cursor shape, not a colour"),
    ("prompt-command-cursor-colour", "the cursor is the terminal's"),
    ("prompt-command-cursor-style", "a cursor shape, not a colour"),
    ("clock-mode-style", "12 or 24 hours, not a colour"),
    ("copy-mode-current-line-style", "unset by default; a style turns a highlight on"),
    ("copy-mode-position-style", "follows mode-style"),
    ("copy-mode-selection-style", "follows mode-style"),
    ("tree-mode-selection-style", "follows mode-style"),
];

fn tmux(socket: &str, args: &[&str]) -> String {
    let mut all = vec!["-L", socket, "-f", "/dev/null"];
    all.extend_from_slice(args);
    exec("tmux", &all)
}

/// Option names from `show` output, with array indices dropped.
fn names(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(|name| name.split('[').next().unwrap_or(name).to_owned())
        .collect()
}

fn is_colour_option(name: &str) -> bool {
    name.ends_with("-style")
        || name.ends_with("-colour")
        || name.starts_with("dark-theme-")
        || name.starts_with("light-theme-")
}

/// `set -g NAME "VALUE"` lines from a theme.
fn settings(theme: &str) -> Vec<(String, String)> {
    theme
        .lines()
        .filter_map(|line| line.strip_prefix("set -g "))
        .filter_map(|rest| rest.split_once(' '))
        .map(|(name, value)| (name.to_owned(), value.trim_matches('"').to_owned()))
        .collect()
}

/// Source a file into a fresh server and return what tmux said.
fn source(socket: &str, path: &std::path::Path) -> String {
    tmux(socket, &["kill-server"]);
    let started = tmux(socket, &["new-session", "-d", "-x", "80", "-y", "24"]);
    started + &tmux(socket, &["source-file", &path.to_string_lossy()])
}

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["tmux"]) {
        return Ok(());
    }
    h.note(exec("tmux", &["-V"]).trim());

    for flavour in h.flavours() {
        let id = flavour.identifier;
        let socket = format!("acid-{id}");
        let path = h.dist(&format!("acid-{id}.conf"));
        let theme = settings(&read(&path));

        let said = source(&socket, &path);
        h.check(
            said.trim().is_empty(),
            format!("{id}: tmux sourced the theme without an error{}", if said.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", said.trim())
            }),
        );

        let wrong: Vec<&String> = theme
            .iter()
            .filter(|(name, value)| {
                tmux(&socket, &["show", "-gv", name]).trim().trim_matches('"') != value
            })
            .map(|(name, _)| name)
            .collect();
        h.check(
            wrong.is_empty(),
            format!("{id}: all {} options read back as written{}", theme.len(), if wrong.is_empty() {
                String::new()
            } else {
                format!("; differ: {wrong:?}")
            }),
        );

        // Hex values sit inside conditional formats, where tmux could take a
        // `#` for one of its own escapes. Expanding one shows it did not.
        let border = tmux(&socket, &["display", "-p", "#{E:pane-active-border-style}"]);
        let blue = format!("fg={}", flavour.colors.blue);
        h.check(
            border.trim() == blue,
            format!("{id}: an ordinary active pane's border expands to {blue}{}", if border.trim() == blue {
                String::new()
            } else {
                format!(", not {}", border.trim())
            }),
        );

        let roles: Vec<&str> = flavour
            .colors
            .iter()
            .filter(|color| {
                let option = format!("@acid_{}", color.identifier);
                tmux(&socket, &["show", "-gv", &option]).trim() != color.hex.to_string()
            })
            .map(|color| color.identifier)
            .collect();
        h.check(
            roles.is_empty(),
            format!("{id}: every role is a user option with its palette value{}", if roles.is_empty() {
                String::new()
            } else {
                format!("; wrong: {roles:?}")
            }),
        );

        let mut known: Vec<String> = Vec::new();
        for scope in ["-g", "-gw", "-s"] {
            known.extend(names(&tmux(&socket, &["show", scope])));
        }
        known.retain(|name| is_colour_option(name));
        known.sort();
        known.dedup();
        h.note(format!("{id}: tmux has {} colour options, {} exempt", known.len(), EXEMPT.len()));

        let missing: Vec<&String> = known
            .iter()
            .filter(|name| !EXEMPT.iter().any(|(exempt, _)| exempt == name))
            .filter(|name| !theme.iter().any(|(set, _)| set == *name))
            .collect();
        h.check(
            missing.is_empty(),
            format!("{id}: every colour option tmux has is set or exempt{}", if missing.is_empty() {
                String::new()
            } else {
                format!("; missing {missing:?}")
            }),
        );

        let stale: Vec<&str> = EXEMPT
            .iter()
            .map(|(name, _)| *name)
            .filter(|name| !known.iter().any(|k| k == name))
            .collect();
        h.check(
            stale.is_empty(),
            format!("{id}: every exempt option is one tmux has{}", if stale.is_empty() {
                String::new()
            } else {
                format!("; unknown {stale:?}")
            }),
        );

        tmux(&socket, &["kill-server"]);
    }

    let unknown = scratch("unknown.conf", "set -g nonsense-style \"fg=#ff0000\"\n")?;
    h.check(
        !source("acid-control", &unknown).trim().is_empty(),
        "control: tmux rejects an unknown option",
    );
    let malformed = scratch("malformed.conf", "set -g status-style \"fg=nonsense\"\n")?;
    h.check(
        !source("acid-control", &malformed).trim().is_empty(),
        "control: tmux rejects a malformed style",
    );
    tmux("acid-control", &["kill-server"]);

    Ok(())
}
