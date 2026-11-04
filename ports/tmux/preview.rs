// tmux running for real in Alacritty: a split window, so the active and inactive
// borders show, and a status line holding a current window, the last window and
// one that has flagged activity.

pub fn run(p: &mut Preview) -> anyhow::Result<()> {
    let theme = p.dist(&format!("acid-{}.conf", p.flavour.identifier));
    let ansi = p.root.join("previews/ansi.sh");
    let config = std::env::temp_dir().join("tmux.conf");
    std::fs::write(
        &config,
        format!(
            "source-file {theme}\n\
             set -as terminal-features ',alacritty:RGB'\n\
             set -g base-index 1\n\
             set -g monitor-activity on\n\
             set -g status-left ' #S '\n\
             set -g status-right \
               '#[fg=#{{@acid_aqua}}]#{{pane_current_command}} #[fg=#{{@acid_overlay1}}]· #[fg=#{{@acid_subtext0}}]#H '\n\
             new-session -d -s acid -n colours 'bash {ansi}; exec sleep 600'\n\
             split-window -h -l 34 -t acid:colours 'exec bash --norc -i'\n\
             new-window -d -n logs 'while sleep 1; do date; done'\n\
             new-window -d -n shell 'exec bash --norc -i'\n\
             select-window -t acid:shell\n\
             select-window -t acid:colours\n",
            theme = theme.display(),
            ansi = ansi.display(),
        ),
    )?;

    // Wide enough for the ANSI sample beside a second pane, and tall enough
    // that the status line is not cut off.
    let config = config.to_string_lossy().into_owned();
    p.in_terminal(
        "1100x660",
        120,
        30,
        11,
        &["tmux", "-L", "preview", "-f", &config, "attach", "-t", "acid"],
    )?;
    // Long enough for the logs window to flag activity.
    p.wait(3);
    p.capture_x11()
}
