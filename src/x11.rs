use crate::{atoms::Atoms, rdwm::Effect};
use log::error;
use xcb::{
    x::{self, EventMask, Window},
    Connection, ProtocolError, VoidCookieChecked, Xid,
};

pub struct X11 {
    conn: Connection,
    root: Window,
    atoms: Atoms,
    wm_check_window: Window,
}

impl X11 {
    pub fn new(conn: Connection, root: Window, atoms: Atoms, wm_check_window: Window) -> Self {
        Self {
            conn,
            root,
            atoms,
            wm_check_window,
        }
    }

    pub const fn root(&self) -> Window {
        self.root
    }

    pub const fn wm_check_window(&self) -> Window {
        self.wm_check_window
    }

    pub const fn atoms(&self) -> &Atoms {
        &self.atoms
    }

    pub fn wait_for_event(&self) -> xcb::Result<xcb::Event> {
        self.conn.wait_for_event()
    }

    pub fn apply_effects_unchecked(&self, effects: &[Effect]) {
        for effect in effects {
            self.send_effect_unchecked(effect);
        }

        if let Err(e) = self.flush() {
            error!("Failed to flush X connection: {e:?}");
        }
    }

    pub fn apply_effects_checked(&self, effects: &[Effect]) {
        let mut pending_checks: Vec<(VoidCookieChecked, String)> = Vec::new();

        for effect in effects {
            let effect_dbg = format!("{effect:?}");
            for cookie in self.send_effect_checked(effect) {
                pending_checks.push((cookie, effect_dbg.clone()));
            }
        }

        if let Err(e) = self.flush() {
            error!("Failed to flush X connection: {e:?}");
        }

        for (cookie, effect_dbg) in pending_checks {
            if let Err(e) = self.check_cookie(cookie) {
                error!("X error applying {effect_dbg}: {e:?}");
            }
        }
    }

    pub fn send_effect_unchecked(&self, effect: &Effect) {
        match effect {
            Effect::Map(window) => {
                self.conn.send_request(&x::MapWindow { window: *window });
            }
            Effect::Unmap(window) => {
                self.conn.send_request(&x::UnmapWindow { window: *window });
            }
            Effect::Focus(window) => {
                self.conn.send_request(&x::SetInputFocus {
                    revert_to: x::InputFocus::PointerRoot,
                    focus: *window,
                    time: x::CURRENT_TIME,
                });
            }
            Effect::Configure {
                window,
                x,
                y,
                w,
                h,
                border,
            } => {
                let config_values = [
                    x::ConfigWindow::X(*x),
                    x::ConfigWindow::Y(*y),
                    x::ConfigWindow::Width(*w),
                    x::ConfigWindow::Height(*h),
                    x::ConfigWindow::BorderWidth(*border),
                ];
                self.conn.send_request(&x::ConfigureWindow {
                    window: *window,
                    value_list: &config_values,
                });
            }
            Effect::ConfigurePositionSize { window, x, y, w, h } => {
                let config_values = [
                    x::ConfigWindow::X(*x),
                    x::ConfigWindow::Y(*y),
                    x::ConfigWindow::Width(*w),
                    x::ConfigWindow::Height(*h),
                ];
                self.conn.send_request(&x::ConfigureWindow {
                    window: *window,
                    value_list: &config_values,
                });
            }
            Effect::SetBorder {
                window,
                pixel,
                width,
            } => {
                self.conn.send_request(&x::ChangeWindowAttributes {
                    window: *window,
                    value_list: &[x::Cw::BorderPixel(*pixel)],
                });
                self.conn.send_request(&x::ConfigureWindow {
                    window: *window,
                    value_list: &[x::ConfigWindow::BorderWidth(*width)],
                });
            }
            Effect::SetCardinal32 {
                window,
                atom,
                value,
            } => {
                self.conn.send_request(&x::ChangeProperty {
                    mode: x::PropMode::Replace,
                    window: *window,
                    property: *atom,
                    r#type: x::ATOM_CARDINAL,
                    data: &[*value],
                });
            }
            Effect::SetAtomList {
                window,
                atom,
                values,
            } => {
                self.conn.send_request(&x::ChangeProperty {
                    mode: x::PropMode::Replace,
                    window: *window,
                    property: *atom,
                    r#type: x::ATOM_ATOM,
                    data: values,
                });
            }
            Effect::SetWindowProperty {
                window,
                atom,
                values,
            } => {
                self.conn.send_request(&x::ChangeProperty {
                    mode: x::PropMode::Replace,
                    window: *window,
                    property: *atom,
                    r#type: x::ATOM_WINDOW,
                    data: values,
                });
            }
            Effect::KillClient(window) => {
                self.conn.send_request(&x::KillClient {
                    resource: window.resource_id(),
                });
            }
            Effect::SendWmDelete(window) => {
                let ev = x::ClientMessageEvent::new(
                    *window,
                    self.atoms.wm_protocols,
                    x::ClientMessageData::Data32([
                        self.atoms.wm_delete_window.resource_id(),
                        x::CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                );
                self.conn.send_request(&x::SendEvent {
                    propagate: false,
                    destination: x::SendEventDest::Window(*window),
                    event_mask: x::EventMask::NO_EVENT,
                    event: &ev,
                });
            }
            Effect::GrabKey {
                keycode,
                modifiers,
                grab_window,
            } => {
                self.conn.send_request(&x::GrabKey {
                    owner_events: false,
                    grab_window: *grab_window,
                    modifiers: *modifiers,
                    key: *keycode,
                    pointer_mode: x::GrabMode::Async,
                    keyboard_mode: x::GrabMode::Async,
                });
            }
        }
    }

    pub fn send_effect_checked(&self, effect: &Effect) -> Vec<VoidCookieChecked> {
        match effect {
            Effect::Map(window) => vec![self
                .conn
                .send_request_checked(&x::MapWindow { window: *window })],
            Effect::Unmap(window) => vec![self
                .conn
                .send_request_checked(&x::UnmapWindow { window: *window })],
            Effect::Focus(window) => vec![self.conn.send_request_checked(&x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: *window,
                time: x::CURRENT_TIME,
            })],
            Effect::Configure {
                window,
                x,
                y,
                w,
                h,
                border,
            } => vec![self.configure_window_checked(*window, *x, *y, *w, *h, *border)],
            Effect::ConfigurePositionSize { window, x, y, w, h } => {
                let config_values = [
                    x::ConfigWindow::X(*x),
                    x::ConfigWindow::Y(*y),
                    x::ConfigWindow::Width(*w),
                    x::ConfigWindow::Height(*h),
                ];
                vec![self.conn.send_request_checked(&x::ConfigureWindow {
                    window: *window,
                    value_list: &config_values,
                })]
            }
            Effect::SetBorder {
                window,
                pixel,
                width,
            } => {
                let a = self.conn.send_request_checked(&x::ChangeWindowAttributes {
                    window: *window,
                    value_list: &[x::Cw::BorderPixel(*pixel)],
                });
                let b = self.conn.send_request_checked(&x::ConfigureWindow {
                    window: *window,
                    value_list: &[x::ConfigWindow::BorderWidth(*width)],
                });
                vec![a, b]
            }
            Effect::SetCardinal32 {
                window,
                atom,
                value,
            } => vec![self.conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: *window,
                property: *atom,
                r#type: x::ATOM_CARDINAL,
                data: &[*value],
            })],
            Effect::SetAtomList {
                window,
                atom,
                values,
            } => vec![self.conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: *window,
                property: *atom,
                r#type: x::ATOM_ATOM,
                data: values,
            })],
            Effect::SetWindowProperty {
                window,
                atom,
                values,
            } => vec![self.conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: *window,
                property: *atom,
                r#type: x::ATOM_WINDOW,
                data: values,
            })],
            Effect::KillClient(window) => vec![self.conn.send_request_checked(&x::KillClient {
                resource: window.resource_id(),
            })],
            Effect::SendWmDelete(window) => {
                let ev = x::ClientMessageEvent::new(
                    *window,
                    self.atoms.wm_protocols,
                    x::ClientMessageData::Data32([
                        self.atoms.wm_delete_window.resource_id(),
                        x::CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                );
                vec![self.conn.send_request_checked(&x::SendEvent {
                    propagate: false,
                    destination: x::SendEventDest::Window(*window),
                    event_mask: x::EventMask::NO_EVENT,
                    event: &ev,
                })]
            }
            Effect::GrabKey {
                keycode,
                modifiers,
                grab_window,
            } => vec![self.conn.send_request_checked(&x::GrabKey {
                owner_events: false,
                grab_window: *grab_window,
                modifiers: *modifiers,
                key: *keycode,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
            })],
        }
    }

    pub fn flush(&self) -> xcb::Result<()> {
        self.conn.flush().map_err(Into::into)
    }

    pub fn check_cookie(&self, cookie: VoidCookieChecked) -> xcb::Result<()> {
        self.conn.check_request(cookie).map_err(Into::into)
    }

    pub fn set_root_event_mask(&self) -> Result<(), ProtocolError> {
        let values = [x::Cw::EventMask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::KEY_PRESS,
        )];
        self.conn
            .send_and_check_request(&x::ChangeWindowAttributes {
                window: self.root,
                value_list: &values,
            })
    }

    pub fn get_root_window_children(&self) -> Result<Vec<Window>, xcb::Error> {
        let cookie = self.conn.send_request(&x::QueryTree { window: self.root });
        let reply = self.conn.wait_for_reply(cookie)?;
        Ok(reply.children().to_vec())
    }

    pub fn is_dock_window(&self, window: Window) -> bool {
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.wm_window_type,
            r#type: x::ATOM_ATOM,
            long_offset: 0,
            long_length: 32,
        });

        if let Ok(reply) = self.conn.wait_for_reply(cookie) {
            let atoms_vec: &[x::Atom] = reply.value();
            atoms_vec
                .iter()
                .any(|a| a.resource_id() == self.atoms.wm_window_type_dock.resource_id())
        } else {
            false
        }
    }

    pub fn supports_wm_delete(&self, window: Window) -> Result<bool, xcb::Error> {
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.wm_protocols,
            r#type: x::ATOM_ATOM,
            long_offset: 0,
            long_length: 1024,
        });

        let reply = self.conn.wait_for_reply(cookie)?;
        let atoms_list: &[x::Atom] = reply.value();
        Ok(atoms_list.contains(&self.atoms.wm_delete_window))
    }

    pub fn get_cardinal32(&self, window: Window, prop: x::Atom) -> Option<u32> {
        Atoms::get_cardinal32(&self.conn, window, prop)
    }

    fn configure_window_checked(
        &self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border: u32,
    ) -> VoidCookieChecked {
        let config_values = [
            x::ConfigWindow::X(x),
            x::ConfigWindow::Y(y),
            x::ConfigWindow::Width(width),
            x::ConfigWindow::Height(height),
            x::ConfigWindow::BorderWidth(border),
        ];

        self.conn.send_request_checked(&x::ConfigureWindow {
            window,
            value_list: &config_values,
        })
    }
}
