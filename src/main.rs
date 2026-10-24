//! RustSpace daemon entry point.
//!
//! Phase 1 responsibilities:
//!   - Load and merge configuration (built-in defaults + user config)
//!   - Log a summary of the resolved config to stderr
//!   - Enter a daemon loop that can be cleanly interrupted with Ctrl-C
//!
//! Later phases will add:
//!   - AX API window tracking  (ax.rs)
//!   - CGEventTap hotkey layer (hotkeys.rs)
//!   - Layout engine           (layout.rs)
//!   - Virtual workspaces      (workspace.rs)
//!   - Unix-socket IPC         (ipc.rs)

mod config;

use config::load_config;

fn main() {
    // -----------------------------------------------------------------------
    // 1. Load configuration
    // -----------------------------------------------------------------------
    let (cfg, warnings) = match load_config() {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("[rustspaced] FATAL: failed to load config: {e}");
            std::process::exit(1);
        }
    };

    for warn in &warnings {
        eprintln!("[rustspaced] WARNING: {warn}");
    }

    // -----------------------------------------------------------------------
    // 2. Log resolved config summary
    // -----------------------------------------------------------------------
    eprintln!("[rustspaced] Starting (config-version = {})", cfg.config_version);
    eprintln!(
        "[rustspaced] Layout  : {} / {}",
        cfg.default_root_container_layout, cfg.default_root_container_orientation
    );
    eprintln!(
        "[rustspaced] Gaps    : inner h={} v={}  outer l={} r={} t={} b={}",
        cfg.gaps.inner.horizontal,
        cfg.gaps.inner.vertical,
        cfg.gaps.outer.left,
        cfg.gaps.outer.right,
        cfg.gaps.outer.top,
        cfg.gaps.outer.bottom,
    );
    eprintln!(
        "[rustspaced] Keyboard: {}",
        cfg.key_mapping.preset
    );
    eprintln!(
        "[rustspaced] Focus-follows-mouse: {}",
        cfg.focus_follows_mouse.enabled
    );
    eprintln!(
        "[rustspaced] Persistent workspaces ({} total): {}",
        cfg.persistent_workspaces.len(),
        cfg.persistent_workspaces.join(", ")
    );
    eprintln!(
        "[rustspaced] Binding modes: {}",
        cfg.modes.keys().cloned().collect::<Vec<_>>().join(", ")
    );

    // Log binding counts per mode.
    for (mode_name, mode_cfg) in &cfg.modes {
        let binding_summary: Vec<String> = mode_cfg
            .binding
            .iter()
            .map(|(key, val)| {
                let cmds = val.commands().join(" && ");
                format!("  {key} → {cmds}")
            })
            .collect();
        eprintln!("[rustspaced] Mode '{mode_name}' ({} bindings):", mode_cfg.binding.len());
        for s in &binding_summary {
            eprintln!("[rustspaced]   {s}");
        }
    }

    // -----------------------------------------------------------------------
    // 3. Workspace state (Phase 1: simple list, no AX yet)
    // -----------------------------------------------------------------------
    let mut workspaces: Vec<Workspace> = cfg
        .persistent_workspaces
        .iter()
        .map(|name| Workspace::new(name.clone()))
        .collect();

    // Ensure at least workspace "1" exists.
    if workspaces.is_empty() {
        workspaces.push(Workspace::new("1".to_string()));
    }

    // These will be mutated in Phase 4 (hotkey/IPC dispatch).
    #[allow(unused_mut)]
    let mut active_workspace_idx: usize = 0;
    #[allow(unused_mut)]
    let mut active_mode: String = "main".to_string();

    eprintln!(
        "[rustspaced] Initial workspace: '{}'  mode: '{}'",
        workspaces[active_workspace_idx].name,
        active_mode
    );

    // -----------------------------------------------------------------------
    // 4. Daemon loop
    //
    // Phase 1: a simple sleep loop.  SIGINT / Ctrl-C exits via the default
    // signal handler — no custom signal handling needed yet.
    //
    // Phase 4 will replace this loop body with:
    //   - Draining a channel fed by the CGEventTap thread (hotkeys)
    //   - Draining a channel fed by the IPC Unix-socket server
    //   - Calling into layout.rs to apply any pending re-layouts
    // -----------------------------------------------------------------------
    eprintln!("[rustspaced] Daemon loop started. Press Ctrl-C to stop.");

    loop {
        // TODO (Phase 2): poll AX observer events
        // TODO (Phase 4): drain hotkey event channel
        // TODO (Phase 1b): drain IPC command channel

        std::thread::sleep(std::time::Duration::from_millis(100));

        // Placeholder: command dispatch table (will be driven by IPC/hotkeys)
        let _dispatch = |cmd: &str, ws: &mut Vec<Workspace>, idx: &mut usize, mode: &mut String| {
            dispatch_command(cmd, ws, idx, mode, &cfg.modes);
        };
    }
}

// ---------------------------------------------------------------------------
// Workspace (Phase 1 — data only, no AX window references yet)
// ---------------------------------------------------------------------------

/// A virtual workspace.  Phase 1 carries just a name; later phases add the
/// layout tree and lists of tracked windows.
struct Workspace {
    name: String,
}

impl Workspace {
    fn new(name: String) -> Self {
        Workspace { name }
    }
}

// ---------------------------------------------------------------------------
// Command dispatcher (stub)
// ---------------------------------------------------------------------------

/// Parses and dispatches a command string to the appropriate handler.
///
/// Phase 1 only handles `workspace <name>` and `mode <name>`.  All other
/// commands are logged as unimplemented.
fn dispatch_command(
    cmd: &str,
    workspaces: &mut Vec<Workspace>,
    active_idx: &mut usize,
    active_mode: &mut String,
    modes: &std::collections::HashMap<String, config::ModeConfig>,
) {
    let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
    match parts.as_slice() {
        ["workspace", name] => {
            if let Some(idx) = workspaces.iter().position(|w| w.name == *name) {
                *active_idx = idx;
                eprintln!("[rustspaced] Switched to workspace '{name}'");
            } else {
                eprintln!("[rustspaced] Unknown workspace '{name}'");
            }
        }
        ["mode", name] => {
            if modes.contains_key(*name) {
                *active_mode = name.to_string();
                eprintln!("[rustspaced] Switched to mode '{name}'");
            } else {
                eprintln!("[rustspaced] Unknown mode '{name}'");
            }
        }
        ["reload-config"] => {
            eprintln!("[rustspaced] TODO: reload-config (Phase 1b)");
        }
        [cmd, ..] => {
            eprintln!("[rustspaced] Unimplemented command: '{cmd}' (scheduled for later phase)");
        }
        [] => {}
    }
}

// ---------------------------------------------------------------------------
// Tests — validate the 4 workspace use cases from USE-CASES.md
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use config::{load_config, BindingValue};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build the minimal workspace list used by every test (workspaces 1-9).
    fn make_workspaces() -> Vec<Workspace> {
        (1..=9).map(|i| Workspace::new(i.to_string())).collect()
    }

    /// Load the default config (no user config present during tests) and
    /// return it.  Panics if loading fails — that is itself a test failure.
    fn default_cfg() -> config::Config {
        let (cfg, _warnings) = load_config().expect("default config must load");
        cfg
    }

    // -----------------------------------------------------------------------
    // Config-level tests (UC-1 … UC-4 traceability table)
    // -----------------------------------------------------------------------

    /// UC-1 / UC-2: default config must bind alt-1 → "workspace 1"
    ///              and alt-2 → "workspace 2".
    #[test]
    fn default_config_has_workspace_switch_bindings() {
        let cfg = default_cfg();
        let main_mode = cfg.modes.get("main").expect("main mode must exist");

        // UC-1
        let cmd1 = main_mode.binding.get("alt-1")
            .expect("alt-1 binding must exist");
        assert_eq!(
            cmd1.commands(),
            &["workspace 1"],
            "UC-1: alt-1 should map to 'workspace 1'"
        );

        // UC-2
        let cmd2 = main_mode.binding.get("alt-2")
            .expect("alt-2 binding must exist");
        assert_eq!(
            cmd2.commands(),
            &["workspace 2"],
            "UC-2: alt-2 should map to 'workspace 2'"
        );
    }

    /// UC-3 / UC-4: default config must bind alt-shift-1 →
    /// "move-node-to-workspace 1" and alt-shift-2 → "move-node-to-workspace 2".
    #[test]
    fn default_config_has_move_node_bindings() {
        let cfg = default_cfg();
        let main_mode = cfg.modes.get("main").expect("main mode must exist");

        // UC-3
        let cmd3 = main_mode.binding.get("alt-shift-1")
            .expect("alt-shift-1 binding must exist");
        assert_eq!(
            cmd3.commands(),
            &["move-node-to-workspace 1"],
            "UC-3: alt-shift-1 should map to 'move-node-to-workspace 1'"
        );

        // UC-4
        let cmd4 = main_mode.binding.get("alt-shift-2")
            .expect("alt-shift-2 binding must exist");
        assert_eq!(
            cmd4.commands(),
            &["move-node-to-workspace 2"],
            "UC-4: alt-shift-2 should map to 'move-node-to-workspace 2'"
        );
    }

    // -----------------------------------------------------------------------
    // Dispatch-level tests
    // -----------------------------------------------------------------------

    /// UC-1: dispatching "workspace 1" from workspace 2 lands on workspace 1.
    #[test]
    fn uc1_dispatch_workspace_1() {
        let cfg = default_cfg();
        let mut ws = make_workspaces();
        let mut idx: usize = 1; // start on workspace "2" (index 1)
        let mut mode = "main".to_string();

        // Simulate: binding lookup yields "workspace 1", then dispatch.
        let cmd = cfg.modes["main"].binding["alt-1"].commands()[0].clone();
        assert_eq!(cmd, "workspace 1");

        dispatch_command(&cmd, &mut ws, &mut idx, &mut mode, &cfg.modes);

        assert_eq!(ws[idx].name, "1", "UC-1: active workspace must be '1'");
        assert_eq!(mode, "main", "UC-1: mode must remain 'main'");
    }

    /// UC-2: dispatching "workspace 2" from workspace 1 lands on workspace 2.
    #[test]
    fn uc2_dispatch_workspace_2() {
        let cfg = default_cfg();
        let mut ws = make_workspaces();
        let mut idx: usize = 0; // start on workspace "1" (index 0)
        let mut mode = "main".to_string();

        let cmd = cfg.modes["main"].binding["alt-2"].commands()[0].clone();
        assert_eq!(cmd, "workspace 2");

        dispatch_command(&cmd, &mut ws, &mut idx, &mut mode, &cfg.modes);

        assert_eq!(ws[idx].name, "2", "UC-2: active workspace must be '2'");
        assert_eq!(mode, "main", "UC-2: mode must remain 'main'");
    }

    /// UC-3: dispatching "move-node-to-workspace 1" does NOT change the
    ///       active workspace index.
    #[test]
    fn uc3_dispatch_move_node_to_workspace_1() {
        let cfg = default_cfg();
        let mut ws = make_workspaces();
        let mut idx: usize = 1; // on workspace "2"
        let mut mode = "main".to_string();

        let cmd = cfg.modes["main"].binding["alt-shift-1"].commands()[0].clone();
        assert_eq!(cmd, "move-node-to-workspace 1");

        dispatch_command(&cmd, &mut ws, &mut idx, &mut mode, &cfg.modes);

        // Active workspace must NOT change — the window moved, not the user.
        assert_eq!(ws[idx].name, "2", "UC-3: active workspace must stay '2'");
        assert_eq!(mode, "main", "UC-3: mode must remain 'main'");
    }

    /// UC-4: dispatching "move-node-to-workspace 2" does NOT change the
    ///       active workspace index.
    #[test]
    fn uc4_dispatch_move_node_to_workspace_2() {
        let cfg = default_cfg();
        let mut ws = make_workspaces();
        let mut idx: usize = 0; // on workspace "1"
        let mut mode = "main".to_string();

        let cmd = cfg.modes["main"].binding["alt-shift-2"].commands()[0].clone();
        assert_eq!(cmd, "move-node-to-workspace 2");

        dispatch_command(&cmd, &mut ws, &mut idx, &mut mode, &cfg.modes);

        // Active workspace must NOT change.
        assert_eq!(ws[idx].name, "1", "UC-4: active workspace must stay '1'");
        assert_eq!(mode, "main", "UC-4: mode must remain 'main'");
    }

    // -----------------------------------------------------------------------
    // Extra robustness tests
    // -----------------------------------------------------------------------

    /// Switching to an unknown workspace leaves the index unchanged.
    #[test]
    fn dispatch_unknown_workspace_is_noop() {
        let cfg = default_cfg();
        let mut ws = make_workspaces();
        let mut idx: usize = 0;
        let mut mode = "main".to_string();

        dispatch_command("workspace Z", &mut ws, &mut idx, &mut mode, &cfg.modes);

        assert_eq!(idx, 0, "Unknown workspace must not change active index");
    }

    /// User config overrides a single binding while preserving the rest.
    #[test]
    fn user_config_merge_overrides_single_binding() {
        use config::{parse_raw_for_test, merge_for_test};

        // Simulate a user config that changes alt-1 to go to workspace 9.
        let user_toml = r#"
[mode.main.binding]
alt-1 = "workspace 9"
"#;
        let (base_cfg, _) = load_config().expect("default config must load");
        let raw_user = parse_raw_for_test(user_toml).expect("user toml must parse");
        let merged = merge_for_test(base_cfg, raw_user);

        let cmd = merged.modes["main"].binding["alt-1"].commands()[0].to_string();
        assert_eq!(cmd, "workspace 9", "User override of alt-1 must take effect");

        // alt-2 must be unchanged.
        let cmd2 = merged.modes["main"].binding["alt-2"].commands()[0].to_string();
        assert_eq!(cmd2, "workspace 2", "alt-2 must remain unchanged after partial user override");
    }
}
