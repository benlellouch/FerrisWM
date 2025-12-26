use crate::key_mapping::{ActionEvent, ActionMapping};
use xcb::x::ModMask;
use xkbcommon::xkb;

pub const NUM_WORKSPACES: usize = 10;
pub const DEFAULT_BORDER_WIDTH: u32 = 3;
pub const DEFAULT_WINDOW_GAP: u32 = 0;

const MOD: ModMask = ModMask::N1;

pub static ACTION_MAPPINGS: &[ActionMapping] = &[
    ActionMapping {
        key: xkb::Keysym::Return,
        modifiers: &[MOD],
        action: ActionEvent::Spawn("st"),
    },
    ActionMapping {
        key: xkb::Keysym::Return,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::Spawn("google-chrome-stable"),
    },
    ActionMapping {
        key: xkb::Keysym::space,
        modifiers: &[MOD],
        action: ActionEvent::Spawn("rofi -show drun"),
    },
    ActionMapping {
        key: xkb::Keysym::d,
        modifiers: &[MOD],
        action: ActionEvent::Spawn("xclock"),
    },
    ActionMapping {
        key: xkb::Keysym::q,
        modifiers: &[MOD],
        action: ActionEvent::Kill,
    },
    ActionMapping {
        key: xkb::Keysym::j,
        modifiers: &[MOD],
        action: ActionEvent::PrevWindow,
    },
    ActionMapping {
        key: xkb::Keysym::k,
        modifiers: &[MOD],
        action: ActionEvent::NextWindow,
    },
    ActionMapping {
        key: xkb::Keysym::j,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SwapLeft,
    },
    ActionMapping {
        key: xkb::Keysym::k,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SwapRight,
    },
    ActionMapping {
        key: xkb::Keysym::equal,
        modifiers: &[MOD],
        action: ActionEvent::IncreaseWindowWeight(1),
    },
    ActionMapping {
        key: xkb::Keysym::minus,
        modifiers: &[MOD],
        action: ActionEvent::DecreaseWindowWeight(1),
    },
    ActionMapping {
        key: xkb::Keysym::equal,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::IncreaseWindowGap(1),
    },
    ActionMapping {
        key: xkb::Keysym::minus,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::DecreaseWindowGap(1),
    },
    ActionMapping {
        key: xkb::Keysym::_1,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(0),
    },
    ActionMapping {
        key: xkb::Keysym::_2,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(1),
    },
    ActionMapping {
        key: xkb::Keysym::_3,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(2),
    },
    ActionMapping {
        key: xkb::Keysym::_4,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(3),
    },
    ActionMapping {
        key: xkb::Keysym::_5,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(4),
    },
    ActionMapping {
        key: xkb::Keysym::_6,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(5),
    },
    ActionMapping {
        key: xkb::Keysym::_7,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(6),
    },
    ActionMapping {
        key: xkb::Keysym::_8,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(7),
    },
    ActionMapping {
        key: xkb::Keysym::_9,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(8),
    },
    ActionMapping {
        key: xkb::Keysym::_0,
        modifiers: &[MOD],
        action: ActionEvent::GoToWorkspace(9),
    },
    ActionMapping {
        key: xkb::Keysym::_1,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(0),
    },
    ActionMapping {
        key: xkb::Keysym::_2,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(1),
    },
    ActionMapping {
        key: xkb::Keysym::_3,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(2),
    },
    ActionMapping {
        key: xkb::Keysym::_4,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(3),
    },
    ActionMapping {
        key: xkb::Keysym::_5,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(4),
    },
    ActionMapping {
        key: xkb::Keysym::_6,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(5),
    },
    ActionMapping {
        key: xkb::Keysym::_7,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(6),
    },
    ActionMapping {
        key: xkb::Keysym::_8,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(7),
    },
    ActionMapping {
        key: xkb::Keysym::_9,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(8),
    },
    ActionMapping {
        key: xkb::Keysym::_0,
        modifiers: &[MOD, ModMask::SHIFT],
        action: ActionEvent::SendToWorkspace(9),
    },
];
