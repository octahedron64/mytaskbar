use super::*;

pub const VK_LIST: [VIRTUAL_KEY; 13] = [VK_NONAME, VK_OEM_MINUS, VK_OEM_7, VK_OEM_5, VK_OEM_3, VK_OEM_4, VK_OEM_PLUS, VK_OEM_1, VK_OEM_6, VK_OEM_COMMA, VK_OEM_PERIOD, VK_OEM_2, VK_OEM_102];
pub const VK_CHARS: [char; 13] = ['!', '-', '^', '|', '@', '[', ';', ':', ']', ',', '.', '/', '_'];

pub const TASKTRAY_MENU: [PCWSTR; 3] = [w!("ホットキー設定"), w!("ウィンドウリスト:ソート設定"), w!("終了"), ];
pub const TASKTARY_DEFAULT_CAPTION: PCWSTR = w!("ホットキー設定画面にて「 ! 」を指定し動作を設定");

pub const DLG_FV_ST_RENAME: &str = "新しいファイル名(&N):";
pub const DLG_FV_BT_TEXT_APPLY: &str = "設定";
pub const DLG_FV_BT_TEXT_OK: &str = "ＯＫ";
pub const DLG_FV_BT_TEXT_CANCEL: &str = "キャンセル";

pub const DLG_HK_CAP_INPUT_INVALID: PCWSTR = w!("入力不正あり");

pub const DLG_HK_BT_TEXT_ADD: &str = "追加(&A)";
pub const DLG_HK_BT_TEXT_APPLY: &str = "設定(&O)";
pub const DLG_HK_BT_TEXT_CANCEL: &str = "キャンセル";
pub const DLG_HK_BT_TEXT_DEL: &str = "削除";

pub const DLG_HK_DDL_HKKIND_LENGTH: &str = "ウィンドウリストああ";
pub const DLG_HK_CB_HKALT_LENGTH: &str = "SHIFTああ";

pub const DLG_HK_DDL_HKKIND: [PCWSTR; 2] = [w!("ランチャー"), w!("ウィンドウリスト")];
pub const DLG_HK_DDL_MODKEY: [PCWSTR; 3] = [w!("ALT"), w!("SHIFT"), w!("NONE")];

pub const DLG_HK_ST_PROP_CAPTIONS: [&str; 5] = [ "ターゲット(&T)：", "表示：", "アイコンサイズ：", "ウィンドウサイズ：", "システムファイル：" ];
pub const DLG_HK_RB_LIST_ICON: [&str; 2] = [ "リスト", "アイコン" ];
pub const DLG_HK_RB_LARGE_SMALL: [&str; 2] = [ "大", "小" ];
pub const DLG_HK_ST_PROP_SIZEEDIT_CAP: [&str; 3] = [ "W", "× H", "px/icons" ];
pub const DLG_HK_CB_DISP_HIDDEN: &str = "表示";
pub const DLG_HK_ST_PROP_SIZEEDIT_SIZE: &str = "999999";
pub const DLG_HK_ST_PROP_WINTASK_LIST: &str = "ウィンドウタスクリスト";
pub const DLG_HK_ST_PROP_PATH_DESKTOP: &str = "デスクトップ";

pub const POPUP_MENUITEM_PROP: PCWSTR = w!("子フォルダ表示プロパティ");
pub const POPUP_MENUITEM_SORT_RESET: PCWSTR = w!("ソート順リセット");
pub const OBJECTITEM_EMPTY: &str = "（なし）";
