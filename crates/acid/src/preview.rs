//! What a port's preview is given to work with.
//!
//! A preview runs the real program and captures what it draws: terminal ports
//! under a headless X server, Wayland ports under a headless compositor. It runs
//! inside the container, where that program is installed.

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use acid_palette::Flavor;
use anyhow::{Context, Result, bail};

pub const FONT: &str = "JetBrainsMono Nerd Font";

pub struct Preview {
    pub root: PathBuf,
    pub port: String,
    pub flavour: &'static Flavor,
    /// Where the image is written.
    pub out: PathBuf,
    /// The window to capture, once a terminal has been started.
    pub window: Option<String>,
    children: Vec<Child>,
}

impl Preview {
    pub fn new(root: PathBuf, port: String, flavour: &'static Flavor, out: PathBuf) -> Self {
        Self {
            root,
            port,
            flavour,
            out,
            window: None,
            children: Vec::new(),
        }
    }

    /// A file this port publishes.
    pub fn dist(&self, relative: &str) -> PathBuf {
        self.root
            .join("ports")
            .join(&self.port)
            .join("dist")
            .join(relative)
    }

    /// A file another port publishes, for the previews that need a terminal.
    pub fn dist_of(&self, port: &str, relative: &str) -> PathBuf {
        self.root
            .join("ports")
            .join(port)
            .join("dist")
            .join(relative)
    }

    fn spawn(&mut self, program: &str, args: &[&str], env: &[(&str, &str)]) -> Result<()> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (key, value) in env {
            command.env(key, value);
        }
        self.children.push(
            command
                .spawn()
                .with_context(|| format!("cannot start {program}"))?,
        );
        Ok(())
    }

    /// Start a headless X server of the given size.
    pub fn start_x11(&mut self, size: &str) -> Result<()> {
        runtime_dir()?;
        self.spawn("Xvfb", &[":99", "-screen", "0", &format!("{size}x24")], &[])?;
        // Waiting on the socket keeps the image one package smaller than
        // waiting on xdpyinfo would.
        for _ in 0..40 {
            if Path::new("/tmp/.X11-unix/X99").exists() {
                sleep(Duration::from_millis(500));
                unsafe {
                    std::env::set_var("DISPLAY", ":99");
                    std::env::set_var("WINIT_UNIX_BACKEND", "x11");
                }
                return Ok(());
            }
            sleep(Duration::from_millis(200));
        }
        bail!("Xvfb did not start")
    }

    /// Start a headless compositor. A backdrop colour keeps a preview from
    /// being framed in black.
    pub fn start_wayland(&mut self, size: &str, backdrop: Option<&str>) -> Result<()> {
        runtime_dir()?;
        let config = std::env::temp_dir().join("sway.conf");
        std::fs::write(
            &config,
            format!(
                "output HEADLESS-1 resolution {size}{}\ndefault_border none\ngaps inner 0\n",
                backdrop
                    .map(|c| format!(" bg {c} solid_color"))
                    .unwrap_or_default()
            ),
        )?;
        self.spawn(
            "sway",
            &["-c", &config.to_string_lossy()],
            &[
                ("WLR_BACKENDS", "headless"),
                ("WLR_LIBINPUT_NO_DEVICES", "1"),
            ],
        )?;
        for _ in 0..30 {
            if Path::new("/tmp/xdg/wayland-1").exists() {
                sleep(Duration::from_secs(1));
                unsafe { std::env::set_var("WAYLAND_DISPLAY", "wayland-1") };
                return Ok(());
            }
            sleep(Duration::from_millis(200));
        }
        bail!("sway did not start")
    }

    /// Run a command in Alacritty under X11, themed by the Alacritty port.
    pub fn in_terminal(
        &mut self,
        size: &str,
        columns: u32,
        lines: u32,
        font_size: u32,
        program: &[&str],
    ) -> Result<()> {
        self.start_x11(size)?;
        let theme = self.dist_of(
            "alacritty",
            &format!("acid-{}.toml", self.flavour.identifier),
        );
        let config = std::env::temp_dir().join("alacritty.toml");
        std::fs::write(
            &config,
            format!(
                "general.import = [{theme:?}]\n\
                 [font]\nsize = {font_size}\n[font.normal]\nfamily = {FONT:?}\n\
                 [window]\npadding = {{ x = 14, y = 12 }}\n\
                 dimensions = {{ columns = {columns}, lines = {lines} }}\n\
                 [cursor]\nstyle = {{ blinking = \"Off\" }}\n",
                theme = theme.to_string_lossy()
            ),
        )?;

        let mut args = vec!["--config-file", config.to_str().context("bad path")?, "-e"];
        args.extend_from_slice(program);
        self.spawn("alacritty", &args, &[])?;
        sleep(Duration::from_secs(5));

        let found = Command::new("xdotool")
            .args(["search", "--class", "Alacritty"])
            .output()?;
        self.window = String::from_utf8_lossy(&found.stdout)
            .lines()
            .next_back()
            .map(str::to_owned);
        Ok(())
    }

    /// Capture the terminal window rather than the whole root, so the image is
    /// the theme and nothing else.
    pub fn capture_x11(&self) -> Result<()> {
        let window = self.window.clone().unwrap_or_else(|| "root".to_owned());
        let status = Command::new("import")
            .args(["-window", &window])
            .arg(&self.out)
            .status()?;
        if !status.success() {
            bail!("could not capture the window");
        }
        Ok(())
    }

    pub fn capture_wayland(&self) -> Result<()> {
        let status = Command::new("grim").arg(&self.out).status()?;
        if !status.success() {
            bail!("could not capture the output");
        }
        Ok(())
    }

    pub fn wait(&self, seconds: u64) {
        sleep(Duration::from_secs(seconds));
    }

    /// Run a command and wait for it, for the steps that must finish.
    pub fn call(&self, program: &str, args: &[&str]) -> Result<()> {
        Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("cannot run {program}"))?;
        Ok(())
    }

    pub fn spawn_detached(
        &mut self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<()> {
        self.spawn(program, args, env)
    }
}

impl Drop for Preview {
    fn drop(&mut self) {
        for child in &mut self.children {
            let _ = child.kill();
        }
    }
}

fn runtime_dir() -> Result<()> {
    let dir = Path::new("/tmp/xdg");
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    unsafe { std::env::set_var("XDG_RUNTIME_DIR", "/tmp/xdg") };
    Ok(())
}
