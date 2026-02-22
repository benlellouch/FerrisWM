use log::{debug, error, info};
use std::process::Command;
use std::{collections::HashMap, process::Stdio};

use xcb::{
    Connection,
    x::{self, ModMask, Window},
};

use crate::atoms::Atoms;
use crate::config::{
    DEFAULT_BORDER_WIDTH, DEFAULT_DOCK_HEIGHT, DEFAULT_WINDOW_GAP, NUM_WORKSPACES,
};
use crate::effect::{Effect, Effects};
use crate::ewmh_manager::EwmhManager;
use crate::key_mapping::ActionEvent;
use crate::keyboard::{fetch_keyboard_mapping, populate_key_bindings};
use crate::state::{ScreenConfig, State};
use crate::x11::{WindowType, X11};

pub struct WindowManager {
    x11: X11,
    ewmh: EwmhManager,
    key_bindings: HashMap<(u8, ModMask), ActionEvent>,
    state: State,
}

impl WindowManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, _) = Connection::connect(None)?;
        info!("Connected to X.");

        let key_bindings = Self::setup_key_bindings(&conn);
        let (screen, root_window) = Self::setup_root(&conn);
        let atoms = Atoms::intern_all(&conn).expect("Failed to intern Atoms");

        // Create WM check window
        let wm_check_window = Self::create_wm_check_window(&conn, root_window);
        let x11 = X11::new(conn, root_window, atoms);
        let ewmh = EwmhManager::new(atoms, root_window, wm_check_window);

        let state = State::new(
            screen,
            DEFAULT_BORDER_WIDTH,
            DEFAULT_WINDOW_GAP,
            DEFAULT_DOCK_HEIGHT,
        );

        let wm = Self {
            x11,
            ewmh,
            key_bindings,
            state,
        };

        wm.x11.set_root_event_mask()?;
        info!("Successfully set substructure redirect");

        // Key grabs
        let keygrab_effects = wm.keygrab_effects();
        wm.x11.apply_effects_checked(&keygrab_effects);

        // EWMH hints
        let ewmh_effects = wm.ewmh.publish_hints();
        wm.x11.apply_effects_unchecked(&ewmh_effects);

        // Publish geometry now that we know screen size, then sync full EWMH state.
        let screen = wm.state.screen();
        let mut ewmh_runtime_effects =
            vec![wm.ewmh.desktop_geometry_effect(screen.width, screen.height)];
        ewmh_runtime_effects.extend(wm.ewmh_sync_effects());
        wm.x11.apply_effects_unchecked(&ewmh_runtime_effects);

        Ok(wm)
    }

    fn ewmh_sync_effects(&self) -> Effects {
        let ewmh = &self.ewmh;
        let screen = self.state.screen();

        let client_list = self.state.client_list_windows();
        let managed = self.state.managed_windows_sorted();

        let mut effects = Vec::new();
        effects.extend(ewmh.client_list_effects(&client_list));
        effects.push(ewmh.current_desktop_effect(self.state.current_workspace_id()));
        effects.push(ewmh.active_window_effect(self.state.focused_window()));
        effects.push(ewmh.workarea_effect(0, 0, screen.width, self.state.usable_screen_height()));

        for window in managed {
            if let Some(workspace) = self.state.window_workspace(window) {
                effects.push(ewmh.window_desktop_effect(window, workspace as u32));
            }
            effects.push(
                ewmh.window_fullscreen_state_effect(
                    window,
                    self.state.is_window_fullscreen(window),
                ),
            );
        }

        effects
    }

    fn setup_key_bindings(conn: &Connection) -> HashMap<(u8, ModMask), ActionEvent> {
        let (keysyms, keysyms_per_keycode) = fetch_keyboard_mapping(conn);
        populate_key_bindings(conn, &keysyms, keysyms_per_keycode)
    }

    fn keygrab_effects(&self) -> Effects {
        let mut effects = Vec::with_capacity(self.key_bindings.len());
        for &(keycode, modifiers) in self.key_bindings.keys() {
            effects.push(Effect::GrabKey {
                keycode,
                modifiers,
                grab_window: self.x11.root(),
            });
        }
        effects
    }

    fn setup_root(conn: &Connection) -> (ScreenConfig, Window) {
        let root = conn.get_setup().roots().next().expect("Cannot find root");
        let screen = ScreenConfig {
            width: u32::from(root.width_in_pixels()),
            height: u32::from(root.height_in_pixels()),
            focused_border_pixel: root.white_pixel(),
            normal_border_pixel: root.black_pixel(),
        };
        (screen, root.root())
    }

    fn create_wm_check_window(conn: &Connection, root: Window) -> Window {
        let win = conn.generate_id();
        let values = [x::Cw::OverrideRedirect(true)];
        conn.send_request(&x::CreateWindow {
            depth: 0,
            wid: win,
            parent: root,
            x: -1,
            y: -1,
            width: 1,
            height: 1,
            border_width: 0,
            class: x::WindowClass::InputOnly,
            visual: 0,
            value_list: &values,
        });
        win
    }

    fn spawn_client(&self, cmd: &str) {
        info!("Spawning command: {cmd}");
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            error!("Empty command provided");
            return;
        }

        let mut command = Command::new(parts[0]);
        for arg in &parts[1..] {
            command.arg(arg);
        }

        match command.spawn() {
            Ok(_) => info!("Successfully spawned: {cmd}"),
            Err(e) => error!("Failed to spawn {cmd}: {e:?}"),
        }
    }

    fn spawn_autostart() {
        match Command::new("sh")
            .arg("-c")
            .arg("exec ~/.config/ferriswm/autostart.sh")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => debug!("Ran autostart succesfully!"),
            Err(e) => debug!("Failed to run autostart: {e:?}"),
        }
    }

    fn close_window(&self, window: Window) -> Effects {
        match self.x11.supports_wm_delete(window) {
            Ok(true) => vec![Effect::SendWmDelete(window)],
            Ok(false) => vec![Effect::KillClient(window)],
            Err(e) => {
                error!(
                    "Failed to query WM_PROTOCOLS for {window:?}: {e:?}. Falling back to force kill."
                );
                vec![Effect::KillClient(window)]
            }
        }
    }

    fn handle_key_press(&mut self, ev: &x::KeyPressEvent) -> Effects {
        let keycode = ev.detail();
        let modifiers = ModMask::from_bits_truncate(ev.state().bits());

        let Some(action) = self.key_bindings.get(&(keycode, modifiers)) else {
            error!("No binding found for keycode: {keycode} with modifiers: {modifiers:?}");
            return vec![];
        };

        match action {
            ActionEvent::Spawn(cmd) => {
                self.spawn_client(cmd);
                vec![]
            }
            ActionEvent::Kill => {
                let Some(window) = self.state.focused_window() else {
                    return vec![];
                };

                self.close_window(window)
            }
            _ => {
                let mut effects = self.state.apply_action(*action);
                effects.extend(self.ewmh_sync_effects());
                effects
            }
        }
    }

    fn handle_client_message(&mut self, ev: &x::ClientMessageEvent) -> Effects {
        let atoms = self.x11.atoms();
        let msg_type = ev.r#type();

        let data32 = match ev.data() {
            x::ClientMessageData::Data32(d) => d,
            _ => return vec![],
        };

        if msg_type == atoms.current_desktop {
            let mut effects = self.state.go_to_workspace(data32[0] as usize);
            effects.extend(self.ewmh_sync_effects());
            return effects;
        }

        if msg_type == atoms.active_window {
            let target = ev.window();
            let desktop_hint = self
                .ewmh
                .get_window_desktop(&self.x11, target)
                .map(|d| d as usize);
            let mut effects = self.state.focus_window(target, desktop_hint);
            effects.extend(self.ewmh_sync_effects());
            return effects;
        }

        if msg_type == atoms.close_window {
            let target = ev.window();
            return self.close_window(target);
        }

        vec![]
    }

    fn grab_windows(&mut self) -> Effects {
        let mut effects = Vec::new();

        match self.x11.get_root_window_children() {
            Ok(children) => {
                debug!("Startup scan: {} root children", children.len());
                for window in children {
                    match self.x11.classify_window(window) {
                        WindowType::Dock => {
                            self.state.track_startup_dock(window);
                        }
                        WindowType::Managed => {
                            if let Some(workspace_id) =
                                self.ewmh.get_window_desktop(&self.x11, window)
                                && (workspace_id as usize) < NUM_WORKSPACES
                            {
                                self.state
                                    .track_startup_managed(window, workspace_id as usize);
                            }
                        }
                        WindowType::Unmanaged => {
                            continue;
                        }
                    }
                }
            }
            Err(e) => error!("Failed to grab children of root at startup: {e:?}"),
        }

        let current_desktop = self.ewmh.get_current_desktop(&self.x11).map(|d| d as usize);
        effects.extend(self.state.startup_finalize(current_desktop));
        effects.extend(self.ewmh_sync_effects());
        effects
    }

    pub fn run(&mut self) -> xcb::Result<()> {
        Self::spawn_autostart();
        let startup_effects = self.grab_windows();
        self.x11.apply_effects_unchecked(&startup_effects);

        loop {
            let event = match self.x11.wait_for_event() {
                Ok(ev) => ev,
                Err(xcb::Error::Protocol(e)) => {
                    error!("X11 protocol error: {e:?}");
                    continue;
                }
                Err(e) => return Err(e),
            };

            match event {
                xcb::Event::X(x::Event::KeyPress(ev)) => {
                    debug!("Received KeyPress event: {ev:?}");
                    let effects = self.handle_key_press(&ev);
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::MapRequest(ev)) => {
                    debug!("Received MapRequest event for {:?}", ev.window());
                    let wt = self.x11.classify_window(ev.window());
                    debug!("Window type {wt:?} for window {:?}", ev.window());
                    let mut effects = self.state.on_map_request(ev.window(), wt);
                    effects.extend(self.ewmh_sync_effects());
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::DestroyNotify(ev)) => {
                    debug!("Received DestroyNotify event for  {:?}", ev.window());
                    let mut effects = self.state.on_destroy(ev.window());
                    effects.extend(self.ewmh_sync_effects());
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::UnmapNotify(ev)) => {
                    debug!("Received UnmapNotify event for {:?}", ev.window());
                    let mut effects = self.state.on_unmap(ev.window());
                    effects.extend(self.ewmh_sync_effects());
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::ClientMessage(ev)) => {
                    debug!("Received ClientMessage event: {ev:?}");
                    let effects = self.handle_client_message(&ev);
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::ButtonPress(ev)) => {
                    debug!("Received ButtonPress event for {:?}", ev.event());
                    self.x11.allow_events();
                    let mut effects = self.state.set_focus(ev.event());
                    effects.extend(self.ewmh_sync_effects());
                    self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::EnterNotify(ev)) => {
                    debug!("Received EnterNotify event for {:?}", ev.event());
                    // TODO Enable in config later
                    // let mut effects = self.state.set_focus(ev.event());
                    // effects.extend(self.ewmh_sync_effects());
                    // self.x11.apply_effects_unchecked(&effects);
                }
                xcb::Event::X(x::Event::MapNotify(ev)) => {
                    debug!("Window mapped: {:?}", ev.window());
                }
                ev => {
                    debug!("Ignoring event: {ev:?}");
                }
            }
        }
    }
}

#[cfg(test)]
mod window_manager_tests {
    use super::*;
    use xcb::{Xid, XidNew};

    fn try_make_wm() -> Option<WindowManager> {
        let (conn, _) = Connection::connect(None).ok()?;
        let (screen, root) = WindowManager::setup_root(&conn);
        let atoms = Atoms::intern_all(&conn).ok()?;
        let wm_check_window = WindowManager::create_wm_check_window(&conn, root);

        let x11 = X11::new(conn, root, atoms);
        let ewmh = EwmhManager::new(atoms, root, wm_check_window);
        let state = State::new(
            screen,
            DEFAULT_BORDER_WIDTH,
            DEFAULT_WINDOW_GAP,
            DEFAULT_DOCK_HEIGHT,
        );

        Some(WindowManager {
            x11,
            ewmh,
            key_bindings: HashMap::new(),
            state,
        })
    }

    #[test]
    fn test_keygrab_effects_match_bindings() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        wm.key_bindings
            .insert((10, ModMask::SHIFT), ActionEvent::NextWindow);
        wm.key_bindings
            .insert((20, ModMask::CONTROL), ActionEvent::PrevWindow);

        let effects = wm.keygrab_effects();

        assert_eq!(effects.len(), 2);
        assert!(effects.contains(&Effect::GrabKey {
            keycode: 10,
            modifiers: ModMask::SHIFT,
            grab_window: wm.x11.root(),
        }));
        assert!(effects.contains(&Effect::GrabKey {
            keycode: 20,
            modifiers: ModMask::CONTROL,
            grab_window: wm.x11.root(),
        }));
    }

    #[test]
    fn test_ewmh_sync_effects_include_workarea_and_active_window() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let win1 = Window::new(1);
        let win2 = Window::new(2);
        let second_workspace = if NUM_WORKSPACES > 1 { 1 } else { 0 };

        wm.state.track_startup_managed(win1, 0);
        wm.state.track_startup_managed(win2, second_workspace);
        let _ = wm.state.set_focus(win1);

        wm.state.track_startup_dock(Window::new(99));

        let effects = wm.ewmh_sync_effects();
        let atoms = *wm.x11.atoms();
        let usable_height = wm.state.usable_screen_height();
        let screen = wm.state.screen();

        let mut expected_workarea = Vec::with_capacity(NUM_WORKSPACES * 4);
        for _ in 0..NUM_WORKSPACES {
            expected_workarea.extend_from_slice(&[0, 0, screen.width, usable_height]);
        }

        assert!(effects.contains(&Effect::SetCardinal32List {
            window: wm.x11.root(),
            atom: atoms.workarea,
            values: expected_workarea,
        }));
        assert!(effects.contains(&Effect::SetWindowProperty {
            window: wm.x11.root(),
            atom: atoms.active_window,
            values: vec![win1.resource_id()],
        }));
        assert!(effects.contains(&Effect::SetCardinal32 {
            window: win2,
            atom: atoms.wm_desktop,
            value: second_workspace as u32,
        }));
        assert!(effects.contains(&Effect::SetAtomList {
            window: win1,
            atom: atoms.wm_state,
            values: vec![],
        }));
    }

    #[test]
    fn test_handle_client_message_current_desktop_updates_state() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let atoms = *wm.x11.atoms();
        let target = if NUM_WORKSPACES > 1 { 1 } else { 0 };

        let ev = x::ClientMessageEvent::new(
            wm.x11.root(),
            atoms.current_desktop,
            x::ClientMessageData::Data32([target as u32, 0, 0, 0, 0]),
        );

        let effects = wm.handle_client_message(&ev);

        assert_eq!(wm.state.current_workspace_id(), target);
        assert!(effects.contains(&Effect::SetCardinal32 {
            window: wm.x11.root(),
            atom: atoms.current_desktop,
            value: target as u32,
        }));
    }

    #[test]
    fn test_handle_client_message_ignores_unhandled_type() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let win = Window::new(1);
        wm.state.track_startup_managed(win, 0);
        let _ = wm.state.set_focus(win);

        let atoms = *wm.x11.atoms();
        let ev = x::ClientMessageEvent::new(
            wm.x11.root(),
            atoms.wm_name,
            x::ClientMessageData::Data32([0, 0, 0, 0, 0]),
        );

        let effects = wm.handle_client_message(&ev);

        assert!(effects.is_empty());
        assert_eq!(wm.state.current_workspace_id(), 0);
        assert_eq!(wm.state.focused_window(), Some(win));
    }

    #[test]
    fn test_handle_client_message_non_data32_returns_empty() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let atoms = *wm.x11.atoms();
        let ev = x::ClientMessageEvent::new(
            wm.x11.root(),
            atoms.current_desktop,
            x::ClientMessageData::Data8([0; 20]),
        );

        let effects = wm.handle_client_message(&ev);
        assert!(effects.is_empty());
    }

    #[test]
    fn test_handle_client_message_active_window_focuses() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let win1 = Window::new(1);
        let win2 = Window::new(2);
        wm.state.track_startup_managed(win1, 0);
        wm.state.track_startup_managed(win2, 0);
        let _ = wm.state.set_focus(win1);

        let atoms = *wm.x11.atoms();
        let ev = x::ClientMessageEvent::new(
            win2,
            atoms.active_window,
            x::ClientMessageData::Data32([0, 0, 0, 0, 0]),
        );

        let effects = wm.handle_client_message(&ev);

        assert_eq!(wm.state.focused_window(), Some(win2));
        assert!(effects.contains(&Effect::Focus(win2)));
    }

    #[test]
    fn test_handle_client_message_close_window_kills_client() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        // Window::new(42) doesn't exist on the X server, so
        // supports_wm_delete will error → fallback to KillClient.
        let target = Window::new(42);
        let atoms = *wm.x11.atoms();
        let ev = x::ClientMessageEvent::new(
            target,
            atoms.close_window,
            x::ClientMessageData::Data32([0, 0, 0, 0, 0]),
        );

        let effects = wm.handle_client_message(&ev);
        assert!(effects.contains(&Effect::KillClient(target)));
    }

    #[test]
    fn test_close_window_fallback_to_kill_on_error() {
        let wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        // A non-existent window triggers the Err branch in close_window.
        let fake = Window::new(999);
        let effects = wm.close_window(fake);
        assert_eq!(effects, vec![Effect::KillClient(fake)]);
    }

    #[test]
    fn test_keygrab_effects_empty_bindings() {
        let wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        // try_make_wm creates an empty key_bindings map.
        let effects = wm.keygrab_effects();
        assert!(effects.is_empty());
    }

    #[test]
    fn test_ewmh_sync_effects_no_windows() {
        let wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let effects = wm.ewmh_sync_effects();
        let atoms = *wm.x11.atoms();

        // Should still contain client_list, current_desktop, active_window, workarea.
        assert!(effects.contains(&Effect::SetWindowProperty {
            window: wm.x11.root(),
            atom: atoms.client_list,
            values: vec![],
        }));
        assert!(effects.contains(&Effect::SetWindowProperty {
            window: wm.x11.root(),
            atom: atoms.active_window,
            values: vec![],
        }));
        assert!(effects.contains(&Effect::SetCardinal32 {
            window: wm.x11.root(),
            atom: atoms.current_desktop,
            value: 0,
        }));
    }

    #[test]
    fn test_ewmh_sync_effects_fullscreen_window() {
        let mut wm = match try_make_wm() {
            Some(wm) => wm,
            None => return,
        };

        let win = Window::new(1);
        wm.state.track_startup_managed(win, 0);
        let _ = wm.state.set_focus(win);
        let _ = wm.state.toggle_fullscreen();

        assert!(wm.state.is_window_fullscreen(win));

        let effects = wm.ewmh_sync_effects();
        let atoms = *wm.x11.atoms();

        assert!(effects.contains(&Effect::SetAtomList {
            window: win,
            atom: atoms.wm_state,
            values: vec![atoms.wm_state_fullscreen.resource_id()],
        }));
    }
}
