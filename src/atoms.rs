use xcb::atoms_struct;

atoms_struct! {
    #[derive(Copy, Clone, Debug)]
    pub struct Atoms {
        pub number_of_desktops => b"_NET_NUMBER_OF_DESKTOPS" only_if_exists = false,
        pub current_desktop => b"_NET_CURRENT_DESKTOP" only_if_exists = false,
        pub supported => b"_NET_SUPPORTED" only_if_exists = false,
        pub supporting_wm_check => b"_NET_SUPPORTING_WM_CHECK" only_if_exists = false,
        pub wm_window_type => b"_NET_WM_WINDOW_TYPE" only_if_exists = false,
        pub wm_window_type_dock => b"_NET_WM_WINDOW_TYPE_DOCK" only_if_exists = false,
        pub wm_protocols => b"WM_PROTOCOLS" only_if_exists = false,
        pub wm_delete_window => b"WM_DELETE_WINDOW" only_if_exists = false,
        pub wm_desktop => b"_NET_WM_DESKTOP" only_if_exists = false,
    }
}
