use crate::{atoms::Atoms, rdwm::Effect};
use xcb::{
    x::{self, EventMask, Window},
    Connection, ProtocolError, Xid,
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

    pub fn apply(&self, fx: Effect) -> xcb::Result<()> {
        match fx {
            Effect::Map(w) => {
                self.conn
                    .send_and_check_request(&x::MapWindow { window: w })?;
            }
            Effect::Unmap(w) => {
                self.conn
                    .send_and_check_request(&x::UnmapWindow { window: w })?;
            }
            Effect::Focus(w) => {
                self.conn.send_and_check_request(&x::SetInputFocus {
                    revert_to: x::InputFocus::PointerRoot,
                    focus: w,
                    time: x::CURRENT_TIME,
                })?;
            }
            Effect::Configure {
                window,
                x,
                y,
                w,
                h,
                border,
            } => {
                self.configure_window(window, x, y, w, h, border)?;
            }
            Effect::ConfigurePositionSize { window, x, y, w, h } => {
                let config_values = [
                    x::ConfigWindow::X(x),
                    x::ConfigWindow::Y(y),
                    x::ConfigWindow::Width(w),
                    x::ConfigWindow::Height(h),
                ];
                self.conn.send_and_check_request(&x::ConfigureWindow {
                    window,
                    value_list: &config_values,
                })?;
            }
            Effect::SetBorder {
                window,
                pixel,
                width,
            } => {
                self.conn
                    .send_and_check_request(&x::ChangeWindowAttributes {
                        window,
                        value_list: &[x::Cw::BorderPixel(pixel)],
                    })?;

                self.conn.send_and_check_request(&x::ConfigureWindow {
                    window,
                    value_list: &[x::ConfigWindow::BorderWidth(width)],
                })?;
            }
            Effect::SetCardinal32 {
                window,
                atom,
                value,
            } => {
                Atoms::set_cardinal32(&self.conn, window, atom, &[value]);
            }
            Effect::SetAtomList {
                window,
                atom,
                values,
            } => {
                Atoms::set_atom(&self.conn, window, atom, &values);
            }
            Effect::SetWindowProperty {
                window,
                atom,
                values,
            } => {
                Atoms::set_window_property(&self.conn, window, atom, &values);
            }
            Effect::KillClient(window) => {
                self.conn.send_and_check_request(&x::KillClient {
                    resource: window.resource_id(),
                })?;
            }
            Effect::SendWmDelete(window) => {
                let ev = x::ClientMessageEvent::new(
                    window,
                    self.atoms.wm_protocols,
                    x::ClientMessageData::Data32([
                        self.atoms.wm_delete_window.resource_id(),
                        x::CURRENT_TIME,
                        0,
                        0,
                        0,
                    ]),
                );

                self.conn.send_and_check_request(&x::SendEvent {
                    propagate: false,
                    destination: x::SendEventDest::Window(window),
                    event_mask: x::EventMask::NO_EVENT,
                    event: &ev,
                })?;
            }
            Effect::GrabKey {
                keycode,
                modifiers,
                grab_window,
            } => {
                self.conn.send_and_check_request(&x::GrabKey {
                    owner_events: false,
                    grab_window,
                    modifiers,
                    key: keycode,
                    pointer_mode: x::GrabMode::Async,
                    keyboard_mode: x::GrabMode::Async,
                })?;
            }
        }
        Ok(())
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

    fn configure_window(
        &self,
        window: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border: u32,
    ) -> xcb::Result<()> {
        let config_values = [
            x::ConfigWindow::X(x),
            x::ConfigWindow::Y(y),
            x::ConfigWindow::Width(width),
            x::ConfigWindow::Height(height),
            x::ConfigWindow::BorderWidth(border),
        ];

        self.conn.send_and_check_request(&x::ConfigureWindow {
            window,
            value_list: &config_values,
        })?;

        Ok(())
    }
}
