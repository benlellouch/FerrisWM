use crate::key_mapping::{ActionEvent, ActionMapping};
use xcb::x::ModMask;
use xkbcommon::xkb;

pub const NUM_WORKSPACES: usize = 10;
pub const DEFAULT_BORDER_WIDTH: u32 = 5;

pub static ACTION_MAPPINGS: &[ActionMapping] = &[
    ActionMapping {
        key: xkb::Keysym::Return,
        modifiers: &[ModMask::N1],
        action: ActionEvent::Spawn("st"),
    },
    ActionMapping {
        key: xkb::Keysym::Return,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::Spawn("google-chrome-stable"),
    },
    ActionMapping {
        key: xkb::Keysym::d,
        modifiers: &[ModMask::N1],
        action: ActionEvent::Spawn("xclock"),
    },
    ActionMapping {
        key: xkb::Keysym::q,
        modifiers: &[ModMask::N1],
        action: ActionEvent::Kill,
    },
    ActionMapping {
        key: xkb::Keysym::j,
        modifiers: &[ModMask::N1],
        action: ActionEvent::PrevWindow,
    },
    ActionMapping {
        key: xkb::Keysym::k,
        modifiers: &[ModMask::N1],
        action: ActionEvent::NextWindow,
    },
    ActionMapping {
        key: xkb::Keysym::j,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SwapLeft,
    },
    ActionMapping {
        key: xkb::Keysym::k,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SwapRight,
    },
    ActionMapping {
        key: xkb::Keysym::_1,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(0),
    },
    ActionMapping {
        key: xkb::Keysym::_2,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(1),
    },
    ActionMapping {
        key: xkb::Keysym::_3,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(2),
    },
    ActionMapping {
        key: xkb::Keysym::_4,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(3),
    },
    ActionMapping {
        key: xkb::Keysym::_5,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(4),
    },
    ActionMapping {
        key: xkb::Keysym::_6,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(5),
    },
    ActionMapping {
        key: xkb::Keysym::_7,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(6),
    },
    ActionMapping {
        key: xkb::Keysym::_8,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(7),
    },
    ActionMapping {
        key: xkb::Keysym::_9,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(8),
    },
    ActionMapping {
        key: xkb::Keysym::_0,
        modifiers: &[ModMask::N1],
        action: ActionEvent::GoToWorkspace(9),
    },
    ActionMapping {
        key: xkb::Keysym::_1,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(0),
    },
    ActionMapping {
        key: xkb::Keysym::_2,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(1),
    },
    ActionMapping {
        key: xkb::Keysym::_3,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(2),
    },
    ActionMapping {
        key: xkb::Keysym::_4,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(3),
    },
    ActionMapping {
        key: xkb::Keysym::_5,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(4),
    },
    ActionMapping {
        key: xkb::Keysym::_6,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(5),
    },
    ActionMapping {
        key: xkb::Keysym::_7,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(6),
    },
    ActionMapping {
        key: xkb::Keysym::_8,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(7),
    },
    ActionMapping {
        key: xkb::Keysym::_9,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(8),
    },
    ActionMapping {
        key: xkb::Keysym::_0,
        modifiers: &[ModMask::N1, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(9),
    },
];
