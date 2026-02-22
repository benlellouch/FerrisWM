use crate::{atoms::Atoms, effect::Effect};
use log::error;
use xcb::{
    Connection, ProtocolError, VoidCookieChecked, Xid,
    x::{self, EventMask, Window},
};

pub struct X11 {
    conn: Connection,
    root: Window,
    atoms: Atoms,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowType {
    /// Normal client windows the WM should manage (tile/focus/workspace).
    Managed,
    /// Windows that manage themselves (override-redirect popups, menus, tooltips, etc).
    Unmanaged,
    /// Dock/panel windows (EWMH _NET_WM_WINDOW_TYPE_DOCK).
    Dock,
}

/// Generates `_unchecked` and `_checked` method pairs for X11 requests.
///
/// # Syntax
///
/// ```ignore
/// x11_request! {
///     fn method_unchecked / method_checked(&self, arg1: Type1, arg2: Type2)
///     // optional let-bindings for setup:
///     let var = expr;
///     => [request_expr1, request_expr2, ...]
/// }
/// ```
///
/// The macro produces two methods:
/// - The first name: fires each request via `send_request` (returns `()`).
/// - The second name: fires each request via `send_request_checked` and
///   collects the cookies into a `Vec<VoidCookieChecked>`.
macro_rules! x11_request {
    (
        fn $unchecked_name:ident / $checked_name:ident (&$self_:ident $(, $arg:ident : $ty:ty)*)
        $(let $vname:ident = $vinit:expr;)*
        => [$($req:expr),+ $(,)?]
    ) => {
        fn $unchecked_name(&$self_ $(, $arg: $ty)*) {
            $(let $vname = $vinit;)*
            $($self_.conn.send_request(&$req);)+
        }

        fn $checked_name(&$self_ $(, $arg: $ty)*) -> Vec<VoidCookieChecked> {
            $(let $vname = $vinit;)*
            vec![$($self_.conn.send_request_checked(&$req)),+]
        }
    };
}

/// Generates `send_effect_unchecked` and `send_effect_checked` from a single
/// list of `Effect` variant → method mappings, appending the `_unchecked` /
/// `_checked` suffix automatically via `paste`.
macro_rules! effect_dispatch {
    ($self_:ident; $( $pat:pat => $method:ident($($arg:expr),* $(,)?) ),* $(,)?) => {
        paste::paste! {
            pub fn send_effect_unchecked(&$self_, effect: &Effect) {
                match effect {
                    $($pat => $self_.[<$method _unchecked>]($($arg),*),)*
                }
            }

            pub fn send_effect_checked(&$self_, effect: &Effect) -> Vec<VoidCookieChecked> {
                match effect {
                    $($pat => $self_.[<$method _checked>]($($arg),*),)*
                }
            }
        }
    };
}

impl X11 {
    pub fn new(conn: Connection, root: Window, atoms: Atoms) -> Self {
        Self { conn, root, atoms }
    }

    pub const fn root(&self) -> Window {
        self.root
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

    // ── Effect dispatch ─────────────────────────────────────────────────
    // Generates both `send_effect_unchecked` and `send_effect_checked`
    // from a single table of Effect variant → method name mappings.
    effect_dispatch! { self;
        Effect::Map(window)
            => map_window(*window),
        Effect::Unmap(window)
            => unmap_window(*window),
        Effect::Focus(window)
            => focus_window(*window),
        Effect::Raise(window)
            => raise_window(*window),
        Effect::Configure { window, x, y, w, h, border }
            => configure_window(*window, *x, *y, *w, *h, *border),
        Effect::ConfigurePositionSize { window, x, y, w, h }
            => configure_window_position_size(*window, *x, *y, *w, *h),
        Effect::SetBorder { window, pixel, width }
            => set_border(*window, *pixel, *width),
        Effect::SetCardinal32 { window, atom, value }
            => set_cardinal32(*window, *atom, *value),
        Effect::SetCardinal32List { window, atom, values }
            => set_cardinal32_list(*window, *atom, values),
        Effect::SetAtomList { window, atom, values }
            => set_atom_list(*window, *atom, values),
        Effect::SetUtf8String { window, atom, value }
            => set_utf8_string(*window, *atom, value),
        Effect::SetWindowProperty { window, atom, values }
            => set_window_property(*window, *atom, values),
        Effect::KillClient(window)
            => kill_client(*window),
        Effect::SendWmDelete(window)
            => send_wm_delete(*window),
        Effect::GrabKey { keycode, modifiers, grab_window }
            => grab_key(*keycode, *modifiers, *grab_window),
        Effect::GrabButton(window)
            => grab_button(*window),
        Effect::SubscribeEnterNotify(window)
            => subscribe_enter_notify(*window),
    }

    // ── X11 request pairs ───────────────────────────────────────────────
    // Each invocation of `x11_request!` generates both an `_unchecked`
    // and a `_checked` variant of the method.

    x11_request! {
        fn map_window_unchecked / map_window_checked(&self, window: Window)
        => [x::MapWindow { window }]
    }

    x11_request! {
        fn unmap_window_unchecked / unmap_window_checked(&self, window: Window)
        => [x::UnmapWindow { window }]
    }

    x11_request! {
        fn focus_window_unchecked / focus_window_checked(&self, window: Window)
        => [x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        }]
    }

    x11_request! {
        fn raise_window_unchecked / raise_window_checked(&self, window: Window)
        let config_values = [x::ConfigWindow::StackMode(x::StackMode::Above)];
        => [x::ConfigureWindow {
            window,
            value_list: &config_values,
        }]
    }

    x11_request! {
        fn configure_window_unchecked / configure_window_checked(&self, window: Window, x: i32, y: i32, w: u32, h: u32, border: u32)
        let config_values = [
            x::ConfigWindow::X(x),
            x::ConfigWindow::Y(y),
            x::ConfigWindow::Width(w),
            x::ConfigWindow::Height(h),
            x::ConfigWindow::BorderWidth(border),
        ];
        => [x::ConfigureWindow {
            window,
            value_list: &config_values,
        }]
    }

    x11_request! {
        fn configure_window_position_size_unchecked / configure_window_position_size_checked(&self, window: Window, x: i32, y: i32, w: u32, h: u32)
        let config_values = [
            x::ConfigWindow::X(x),
            x::ConfigWindow::Y(y),
            x::ConfigWindow::Width(w),
            x::ConfigWindow::Height(h),
        ];
        => [x::ConfigureWindow {
            window,
            value_list: &config_values,
        }]
    }

    x11_request! {
        fn set_border_unchecked / set_border_checked(&self, window: Window, pixel: u32, width: u32)
        => [
            x::ChangeWindowAttributes {
                window,
                value_list: &[x::Cw::BorderPixel(pixel)],
            },
            x::ConfigureWindow {
                window,
                value_list: &[x::ConfigWindow::BorderWidth(width)],
            },
        ]
    }

    x11_request! {
        fn set_cardinal32_unchecked / set_cardinal32_checked(&self, window: Window, atom: x::Atom, value: u32)
        => [x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atom,
            r#type: x::ATOM_CARDINAL,
            data: &[value],
        }]
    }

    x11_request! {
        fn set_cardinal32_list_unchecked / set_cardinal32_list_checked(&self, window: Window, atom: x::Atom, values: &[u32])
        => [x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atom,
            r#type: x::ATOM_CARDINAL,
            data: values,
        }]
    }

    x11_request! {
        fn set_atom_list_unchecked / set_atom_list_checked(&self, window: Window, atom: x::Atom, values: &[u32])
        => [x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atom,
            r#type: x::ATOM_ATOM,
            data: values,
        }]
    }

    x11_request! {
        fn set_window_property_unchecked / set_window_property_checked(&self, window: Window, atom: x::Atom, values: &[u32])
        => [x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atom,
            r#type: x::ATOM_WINDOW,
            data: values,
        }]
    }

    x11_request! {
        fn set_utf8_string_unchecked / set_utf8_string_checked(&self, window: Window, atom: x::Atom, value: &str)
        let data = value.as_bytes();
        let r#type = self.atoms.utf8_string;
        => [x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atom,
            r#type,
            data,
        }]
    }

    x11_request! {
        fn kill_client_unchecked / kill_client_checked(&self, window: Window)
        let resource = window.resource_id();
        => [x::KillClient { resource }]
    }

    x11_request! {
        fn send_wm_delete_unchecked / send_wm_delete_checked(&self, window: Window)
        let ev = self.wm_delete_client_message(window);
        => [x::SendEvent {
            propagate: false,
            destination: x::SendEventDest::Window(window),
            event_mask: x::EventMask::NO_EVENT,
            event: &ev,
        }]
    }

    x11_request! {
        fn grab_key_unchecked / grab_key_checked(&self, keycode: u8, modifiers: x::ModMask, grab_window: Window)
        => [x::GrabKey {
            owner_events: false,
            grab_window,
            modifiers,
            key: keycode,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
        }]
    }

    x11_request! {
        fn grab_button_unchecked / grab_button_checked(&self, window: Window)
        => [x::GrabButton {
            owner_events: false,
            grab_window: window,
            event_mask: x::EventMask::BUTTON_PRESS,
            pointer_mode: x::GrabMode::Sync,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::WINDOW_NONE,
            cursor: x::CURSOR_NONE,
            button: x::ButtonIndex::N1,
            modifiers: x::ModMask::ANY,
        }]
    }

    x11_request! {
        fn subscribe_enter_notify_unchecked / subscribe_enter_notify_checked(&self, window: Window)
        => [x::ChangeWindowAttributes {
            window,
            value_list: &[x::Cw::EventMask(EventMask::ENTER_WINDOW)],
        }]
    }

    // ── Helpers (not macro-generated) ───────────────────────────────────

    fn wm_delete_client_message(&self, window: Window) -> x::ClientMessageEvent {
        x::ClientMessageEvent::new(
            window,
            self.atoms.wm_protocols,
            x::ClientMessageData::Data32([
                self.atoms.wm_delete_window.resource_id(),
                x::CURRENT_TIME,
                0,
                0,
                0,
            ]),
        )
    }

    pub fn flush(&self) -> xcb::Result<()> {
        self.conn.flush().map_err(Into::into)
    }

    pub fn check_cookie(&self, cookie: VoidCookieChecked) -> xcb::Result<()> {
        self.conn.check_request(cookie).map_err(Into::into)
    }

    pub fn allow_events(&self) {
        self.conn.send_request(&x::AllowEvents {
            mode: x::Allow::ReplayPointer,
            time: x::CURRENT_TIME,
        });
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

    pub fn classify_window(&self, window: Window) -> WindowType {
        // Docks are special-cased: even if override-redirect is set, we want to treat them as docks.
        if self.is_dock_window(window) {
            return WindowType::Dock;
        }

        match self.is_override_redirect(window) {
            Ok(true) => WindowType::Unmanaged,
            Ok(false) => WindowType::Managed,
            // Preserve existing behavior: on query failure, treat as manageable.
            Err(_e) => WindowType::Managed,
        }
    }

    fn is_override_redirect(&self, window: Window) -> Result<bool, xcb::Error> {
        let cookie = self.conn.send_request(&x::GetWindowAttributes { window });
        let reply = self.conn.wait_for_reply(cookie)?;
        Ok(reply.override_redirect())
    }

    fn is_dock_window(&self, window: Window) -> bool {
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

    pub fn get_cardinal32(&self, window: x::Window, prop: x::Atom) -> Option<u32> {
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: prop,
            r#type: x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 1,
        });

        if let Ok(reply) = self.conn.wait_for_reply(cookie) {
            let value: &[u32] = reply.value();
            if !value.is_empty() {
                return value.first().cloned();
            }
        }
        error!("Failed to get Cardinal32 property for atom {prop:?} on {window:?}");
        None
    }
}
