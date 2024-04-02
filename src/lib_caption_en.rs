use super::*;

pub const VK_LIST: [VIRTUAL_KEY; 12] = [VK_NONAME, VK_OEM_8, VK_OEM_MINUS, VK_OEM_PLUS, VK_OEM_4, VK_OEM_6, VK_OEM_5, VK_OEM_1, VK_OEM_7, VK_OEM_COMMA, VK_OEM_PERIOD, VK_OEM_2];
pub const VK_CHARS: [char; 12] = ['!', '`', '-', '=', '[', ']', '\\', ';', '\'', ',', '.', '/'];

pub const TASKTRAY_MENU: [PCWSTR; 3] = [w!("Hotkey Property"), w!("Window List: Sort Edit"), w!("Quit"), ];
pub const TASKTARY_DEFAULT_CAPTION: PCWSTR = w!("Set [None] + [!] hotkey in Hotkey Property dialog box.");

pub const DLG_FV_ST_RENAME: &str = "New Filename(&N):";
pub const DLG_FV_BT_TEXT_APPLY: &str = "Apply";
pub const DLG_FV_BT_TEXT_OK: &str = "OK";
pub const DLG_FV_BT_TEXT_CANCEL: &str = "Cancel";

pub const DLG_HK_CAP_INPUT_INVALID: PCWSTR = w!("Invalid input.");

pub const DLG_HK_BT_TEXT_ADD: &str = "Add(&A)";
pub const DLG_HK_BT_TEXT_APPLY: &str = "Apply(&O)";
pub const DLG_HK_BT_TEXT_CANCEL: &str = "Cancel";
pub const DLG_HK_BT_TEXT_DEL: &str = "Delete";

pub const DLG_HK_DDL_HKKIND_LENGTH: &str = "Window Listwww";
pub const DLG_HK_CB_HKALT_LENGTH: &str = "SHIFTwww";

pub const DLG_HK_DDL_HKKIND: [PCWSTR; 2] = [w!("Launcher"), w!("Window List")];
pub const DLG_HK_DDL_MODKEY: [PCWSTR; 3] = [w!("ALT"), w!("SHIFT"), w!("NONE")];

pub const DLG_HK_ST_PROP_CAPTIONS: [&str; 5] = [ "Target folder(&T): ", "View: ", "Icon size: ", "Window size: ", "System hidden file: " ];
pub const DLG_HK_RB_LIST_ICON: [&str; 2] = [ "List", "Icon" ];
pub const DLG_HK_RB_LARGE_SMALL: [&str; 2] = [ "Large", "Small" ];
pub const DLG_HK_ST_PROP_SIZEEDIT_CAP: [&str; 3] = [ "W", "× H", "px/icons" ];
pub const DLG_HK_CB_DISP_HIDDEN: &str = "Display enable";
pub const DLG_HK_ST_PROP_SIZEEDIT_SIZE: &str = "999999";
pub const DLG_HK_ST_PROP_WINTASK_LIST: &str = "Window List";
pub const DLG_HK_ST_PROP_PATH_DESKTOP: &str = "Desktop";

pub const POPUP_MENUITEM_PROP: PCWSTR = w!("Subfolder View Property");
pub const POPUP_MENUITEM_SORT_RESET: PCWSTR = w!("Reset File Sort");
pub const OBJECTITEM_EMPTY: &str = "( empty )";
