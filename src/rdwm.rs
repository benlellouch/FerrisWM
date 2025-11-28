use std::collections::HashMap;
use std::process::Command;
use xcb::{
    Connection, ProtocolError, Xid, x::{self, Cw, EventMask, ModMask, Window}
};

use crate::config::ACTION_MAPPINGS;
use crate::key_mapping::ActionEvent;

pub struct WindowManager {
    conn: Connection,
    windows: Vec<Window>,
    focus: usize,
    key_bindings: HashMap<(u8, ModMask), ActionEvent>,
    screen_width: u32,
    screen_height: u32,
}

impl WindowManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, _) = Connection::connect(None)?;
        println!("Connected to X.");

        // Get keyboard mapping from X server
        let (keysyms, keysyms_per_keycode) = if let Ok(keyboard_mapping) =
            conn.wait_for_reply(conn.send_request(&x::GetKeyboardMapping {
                first_keycode: conn.get_setup().min_keycode(),
                count: conn.get_setup().max_keycode() - conn.get_setup().min_keycode() + 1,
            })) {
            let keysyms_per_keycode = keyboard_mapping.keysyms_per_keycode() as usize;
            let keysyms = keyboard_mapping.keysyms().to_vec();
            (keysyms, keysyms_per_keycode)
        } else {
            println!("Failed to get keyboard mapping, using empty keysyms");
            (vec![], 0)
        };


        let mut wm = WindowManager {
            conn,
            windows: vec![],
            focus: 0,
            key_bindings: HashMap::new(),
            screen_width: 0,
            screen_height: 0,
        };

        // Create key bindings HashMap
        for mapping in ACTION_MAPPINGS {
            let modifiers = mapping
                .modifiers
                .iter()
                .copied()
                .reduce(|acc, modkey| acc | modkey)
                .unwrap_or(xcb::x::ModMask::empty());

            // Find keycode for this keysym
            for (i, chunk) in keysyms.chunks(keysyms_per_keycode).enumerate() {
                if chunk.contains(&mapping.key.raw()) {
                    let keycode = wm.conn.get_setup().min_keycode() + i as u8;
                    wm.key_bindings
                        .insert((keycode, modifiers), mapping.action);
                    println!(
                        "Mapped key {:?} (keycode: {}) with modifiers {:?} to action: {:?}",
                        mapping.key, keycode, modifiers, mapping.action
                    );
                    break;
                }
            }
        }

        let root_screen = wm.conn.get_setup().roots().next().unwrap();
        wm.screen_width = root_screen.width_in_pixels() as u32;
        wm.screen_height = root_screen.height_in_pixels() as u32;

        // Get root window and set up substructure redirect
        let root = root_screen.root();
        wm.set_substructure_redirect(root)?;
        println!("Successfully set substructure redirect");

        // Set up key grabs
        wm.set_keygrabs(root);


        Ok(wm)
    }

    fn configure_window(
        &self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), ProtocolError> {
        let config_values = [
            x::ConfigWindow::X(x),
            x::ConfigWindow::Y(y),
            x::ConfigWindow::Width(width),
            x::ConfigWindow::Height(height),
        ];

        self.conn.send_and_check_request(&x::ConfigureWindow {
            window,
            value_list: &config_values,
        })
    }

    fn configure_windows(&self) {
        let window_count = self.windows.len() as u32;
        if window_count == 0 {
            return;
        }

        let window_width = self.screen_width / window_count;
        let window_height = self.screen_height;

        for (i, win) in self.windows.iter().enumerate() {
            let x = i as i32 * window_width as i32;
            match self.configure_window(*win, x, 0, window_width, window_height) {
                Ok(_) => (),
                Err(e) => {
                    println!("Failed to configure window {:?}: {:?}", win, e);
                }
            }
        }
    }

    fn handle_key_press(&mut self, ev: &x::KeyPressEvent) {
        let keycode = ev.detail();
        let modifiers = ModMask::from_bits_truncate(ev.state().bits());

        if let Some(action) = self.key_bindings.get(&(keycode, modifiers)) {
            match action {
                ActionEvent::Spawn(cmd) => self.spawn_client(cmd),
                ActionEvent::KillClient => self.kill_client(),
                ActionEvent::FocusNext => self.shift_focus(1),
                ActionEvent::FocusPrev => self.shift_focus(-1),
                _ => {
                    println!("Action {:?} not implemented yet", action);
                }
            }
        } else {
            println!(
                "No binding found for keycode: {} with modifiers: {:?}",
                keycode, modifiers
            );
        }
    }

    fn spawn_client(&self, cmd: &str) {
        println!("Spawning command: {}", cmd);
        match Command::new(cmd).spawn() {
            Ok(_) => println!("Successfully spawned: {}", cmd),
            Err(e) => println!("Failed to spawn {}: {:?}", cmd, e),
        }
    }

    fn kill_client(&mut self) {
        if self.focus < self.windows.len() {
            let window_to_kill = self.windows.remove(self.focus as usize);
            println!("Killing client window: {:?}", window_to_kill);

            // Send KillClient request
            match self.conn.send_and_check_request(&x::KillClient { resource: window_to_kill.resource_id() }) {
                Ok(_) => println!("Successfully killed window: {:?}", window_to_kill),
                Err(e) => println!("Failed to kill window {:?}: {:?}", window_to_kill, e),
            }

            // Reconfigure remaining windows
            self.configure_windows();
        } else {
            println!("No windows to kill");
        }
    }

    fn shift_focus(&mut self, direction: isize) {
        if self.windows.is_empty() {
            println!("No windows to focus");
            return;
        }

        let window_count = self.windows.len() as isize;
        self.focus = ((self.focus as isize + direction + window_count) % window_count) as usize;
        println!("Focus shifted to window index: {}", self.focus);
    }
        

    fn handle_map_request(&mut self, window: Window) {
        // push new window to list
        self.windows.push(window);

        self.configure_windows();

        match self.conn.send_and_check_request(&x::MapWindow { window }) {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to map window {:?}: {:?}", window, e);
            }
        }
    }

    fn set_substructure_redirect(&self, root: Window) -> Result<(), ProtocolError> {
        let values = [Cw::EventMask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::KEY_PRESS,
        )];
        self.conn
            .send_and_check_request(&x::ChangeWindowAttributes {
                window: root,
                value_list: &values,
            })
    }

    fn set_keygrabs(&self, root: Window) {
        for &(keycode, modifiers) in self.key_bindings.keys() {
            match self.conn.send_and_check_request(&x::GrabKey {
                owner_events: false,
                grab_window: root,
                modifiers,
                key: keycode,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
            }) {
                Ok(_) => println!(
                    "Successfully grabbed key: keycode {} with modifiers {:?}",
                    keycode, modifiers
                ),
                Err(e) => println!("Failed to grab key {}: {:?}", keycode, e),
            }
        }
    }

    pub fn run(&mut self) -> xcb::Result<()> {
        loop {
            match self.conn.wait_for_event()? {
                xcb::Event::X(x::Event::KeyPress(ev)) => {
                    println!("Received KeyPress event: {:?}", ev);
                    self.handle_key_press(&ev);
                }

                xcb::Event::X(x::Event::MapRequest(ev)) => {
                    println!("Received MapRequest event for window: {:?}", ev.window());
                    self.handle_map_request(ev.window());
                }

                xcb::Event::X(x::Event::ConfigureRequest(ev)) => {
                    println!(
                        "Received ConfigureRequest event for window: {:?}",
                        ev.window()
                    );
                    println!("  Parent: {:?}", ev.parent());
                    println!("  Requested position: ({}, {})", ev.x(), ev.y());
                    println!("  Requested size: {}x{}", ev.width(), ev.height());

                    // Check if this is a new window
                    if !self.windows.contains(&ev.window()) {
                        println!("  -> New manageable window detected, treating as MapRequest");
                        self.handle_map_request(ev.window());
                    }
                }

                xcb::Event::X(x::Event::MapNotify(ev)) => {
                    println!("Window mapped: {:?}", ev.window());
                }

                ev => {
                    println!("Ignoring event: {:?}", ev);
                }
            }
        }
    }
}
