// fish reads both forms of the port: a .theme file applied through fish_config,
// and a conf.d script that sets the same variables globally. They must agree.

pub fn run(h: &mut Harness) -> anyhow::Result<()> {
    if !h.require(&["fish"]) {
        return Ok(());
    }

    let root = std::env::temp_dir().join("fishconfig");
    std::fs::create_dir_all(root.join("fish/themes"))?;
    std::fs::create_dir_all(root.join("share"))?;
    for flavour in h.flavours() {
        std::fs::copy(
            h.dist(&format!("themes/Acid {}.theme", flavour.name)),
            root.join(format!("fish/themes/Acid {}.theme", flavour.name)),
        )?;
    }

    let home = root.to_string_lossy().into_owned();
    let share = root.join("share").to_string_lossy().into_owned();
    let choose = |name: &str| {
        exec(
            "env",
            &[
                &format!("HOME={home}"),
                &format!("XDG_CONFIG_HOME={home}"),
                &format!("XDG_DATA_HOME={share}"),
                "fish", "-c", &format!("fish_config theme choose \"{name}\""),
            ],
        )
    };

    for flavour in h.flavours() {
        let applied = choose(&format!("Acid {}", flavour.name));
        h.check(
            !applied.contains("Searched"),
            format!("{}: fish_config applied the theme", flavour.name),
        );

        // The two forms must set identical values for every variable.
        let theme_path = h.dist(&format!("themes/Acid {}.theme", flavour.name));
        let conf_path = h.dist(&format!("conf.d/acid-{}.fish", flavour.identifier));
        let declared = read(&theme_path)
            .lines()
            .filter(|line| line.starts_with("fish_"))
            .count();

        let script = format!(
            r#"source {conf}
            set -l bad 0
            set -l total 0
            for line in (cat '{theme}' | string match -rv '^#|^$')
                set -l parts (string split -m 1 ' ' -- $line)
                set -l var $parts[1]
                set -l want (string trim -- $parts[2])
                set -l got (string join ' ' -- $$var)
                set total (math $total + 1)
                if test "$got" != "$want"
                    echo "MISMATCH $var"
                    set bad (math $bad + 1)
                end
            end
            echo "TOTAL $total BAD $bad""#,
            conf = conf_path.display(),
            theme = theme_path.display(),
        );
        let output = exec("fish", &["--no-config", "-c", &script]);
        let counts: Vec<&str> = output
            .lines()
            .find(|line| line.starts_with("TOTAL"))
            .unwrap_or_default()
            .split_whitespace()
            .collect();
        let total: usize = counts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        let bad: usize = counts.get(3).and_then(|v| v.parse().ok()).unwrap_or(1);

        if bad != 0 {
            h.fail(format!("{}: the two forms disagree on {bad} variable(s)", flavour.identifier));
        } else if total != declared {
            h.fail(format!(
                "{}: compared {total} variables but the theme declares {declared}",
                flavour.identifier
            ));
        } else {
            h.pass(format!(
                "{}: both forms agree on all {total} variables",
                flavour.identifier
            ));
        }
    }

    // fish does not validate colour values, so a bad value proves nothing. A
    // theme that does not exist does fail, which is what makes the applies
    // above meaningful.
    h.check(
        choose("DoesNotExist").contains("Searched"),
        "control: fish rejects a theme that does not exist",
    );

    Ok(())
}
