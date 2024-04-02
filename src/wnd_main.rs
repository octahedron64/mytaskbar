use std::collections::VecDeque;
use fxhash::{FxHashSet, FxHashMap};

use self::{dlg_fileview_prop::FileViewPropWnd, dlg_hotkey_prop::HotkeyPropWnd, wnd_fileview::FileViewWnd, wnd_winview::WindowViewWnd};

use super::*;
use crate::{
    lib_property::{PropertyHolder, HotkeyType},
    lib_shell::ObjectHolder,
    lib_window::WindowInfo,
    dlg_fileview_prop::DlgType,
};

//////////////////////////////////////// COLOR

// ダークモード：cargo build --no-default-features --features "dark en"
#[cfg(feature = "light")]
pub const BRUSH_BACKGROUND: SYS_COLOR_INDEX = COLOR_3DFACE;
#[cfg(feature = "dark")]
pub const BRUSH_BACKGROUND: SYS_COLOR_INDEX = COLOR_WINDOWTEXT;

#[cfg(feature = "light")]
pub const COLOR_TEXT: COLORREF = COLORREF(0x000000);
#[cfg(feature = "dark")]
pub const COLOR_TEXT: COLORREF = COLORREF(0xffffff);

#[cfg(feature = "light")]
pub const COLOR_CURSOR_HIGHLIGHT: COLORREF = COLORREF(0xffffff);
#[cfg(feature = "dark")]
pub const COLOR_CURSOR_HIGHLIGHT: COLORREF = COLORREF(0x404040);

// ライトモード・ダークモード共通
pub const COLOR_CTRL_EDGE: COLORREF = COLORREF(0x404040);
pub const COLOR_HIGHLIGHT_BORDER: COLORREF = COLORREF(0xd0d0d0);
pub const COLOR_SCROLLBAR_BORDER: COLORREF = COLORREF(0xa0a0a0);
pub const COLOR_SCROLLBTN_HIGHLIGHT: COLORREF = COLORREF(0xff4040);
pub const COLOR_FILESORT_BORDER: COLORREF = COLORREF(0xff0000);
pub const COLOR_GROUPBOX: COLORREF = COLORREF(0x808080);
pub const COLOR_GROUPBOX_FREE: COLORREF = COLORREF(0xe0e0d0);

// ダークモードに対応しないダイアログ部分の背景、文字色（固定）
pub const BRUSH_DEFAULT_BACKGROUND: SYS_COLOR_INDEX = COLOR_3DFACE;
pub const COLOR_DEFAULT_TEXT: COLORREF = COLORREF(0x000000);
pub const COLOR_DEFAULT_CURSOR_HIGHLIGHT: COLORREF = COLORREF(0xffffff);

//////////////////////////////////////// LOCAL

const MAIN_WINDOW_CLASS: PCWSTR = if cfg!(debug_assertions) {
    w!("mytaskbar_main_window_class_D")
} else {
    w!("mytaskbar_main_window_class")
};

const TASKTRAY_ICON_TEXT: PCWSTR = if cfg!(debug_assertions) {
    w!("mytaskbar_D")
} else {
    w!("mytaskbar")
};
pub trait ViewWindow {
    fn close(&self);
    fn is_close_blocking(&self) -> bool { return false }
}

pub enum WinSortList {
    IMGFILE(String),
    WILDCARD(String)
}

pub struct MainWnd {
    app: Weak<RefCell<App>>,
    handle: HWND,
    msg_taskbar_restart: u32,

    pub vec_window_items: VecDeque<VecDeque<WindowInfo>>, // ウィンドウリスト
    pub vec_window_sortlist: Vec<WinSortList>, // ウィンドウをプロセスグループでソートする時の順序
    pub hash_window_hide: FxHashSet<isize>, // 現在hideしているHWND一覧
    pub vec_auto_hide: Vec<String>, // autohideするプロセスイメージ名

    // fileviewからプロパティウィンドウへの値引き渡し用
    pub rename_parentpath: String,
    pub rename_filename: String,
    pub lauch_propery_dirpath: String,

    vec_hotkey_idx: Vec<String>, // ホットキー登録したindexと対になるホットキー文字列の配列
    hash_hotkey_params: FxHashMap<String, String>, // K:ホットキー文字列, V:パラメータ文字列
    view_wnd: Option<Box<dyn ViewWindow>>, // 子ウィンドウ(同時に一つ。実態はウィンドウのオブジェクトの弱参照)
    b_last_auto_window: bool, // autowindowは、もう一度ホットキー(AW)を押すと消える挙動にするためのフラグ
}

impl Drop for MainWnd {
    fn drop(&mut self) {
    }
}

pub type MainWndWeak = Weak<MainWnd>;
pub type MainWndRc = Rc<MainWnd>;

impl RcValueRef<MainWnd> for MainWndRc {}

impl MainWnd {
    pub fn init(app:  Weak<RefCell<App>>) -> MainWndWeak {
        let wnd = Rc::new(Self {
            app: app,
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            msg_taskbar_restart: 0u32,

            vec_window_items: VecDeque::<VecDeque<WindowInfo>>::default(),
            vec_window_sortlist: Vec::<WinSortList>::default(),
            hash_window_hide: FxHashSet::<isize>::default(),
            vec_auto_hide: Vec::<String>::default(),

            rename_parentpath: String::default(),
            rename_filename: String::default(),
            lauch_propery_dirpath: String::default(),

            vec_hotkey_idx: Vec::<String>::default(),
            hash_hotkey_params: FxHashMap::<String, String>::default(),
            view_wnd: None,
            b_last_auto_window: false,
        });

        let window_class = MAIN_WINDOW_CLASS;
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpszClassName: window_class,
            lpfnWndProc: Some(wnd_proc::<MainWnd>),
            ..Default::default()
        };
        unsafe { RegisterClassExW(&wc) };

        unsafe { CreateWindowExW(WINDOW_EX_STYLE::default(), window_class, w!("MAIN-WINDOW"),  WS_POPUP,
            0, 0, 0, 0, None, None, None, Some(&wnd as *const _ as _)) };

        Rc::downgrade(&wnd)
    }

    pub fn check_instance() -> Result<HWND> {
        let mut ret: Result<HWND> = Err(Error::OK);
        let myclass = unsafe { MAIN_WINDOW_CLASS.to_string() }?;
        let _ = unsafe { EnumWindows(Some(WindowInfo::enum_window_mine), LPARAM(&mut (&mut ret, &myclass) as *mut _ as _))};
        ret
    }

    pub fn set_view_wnd(&mut self, v: Option<Box<dyn ViewWindow>>) {
        if let Some(c) = &self.view_wnd { c.close(); }
        self.view_wnd = v;
    }

    pub fn notify_icon(&mut self, hwnd: HWND, nim: NOTIFY_ICON_MESSAGE) -> BOOL {
        let param = PropertyHolder::load_notify_icon_param();
        let h = Icon(if nim == NIM_ADD {
            if let Some((path, index)) = param {
                HICON(Icon::get_shell_file_icon(WSTR::from(&path).PCWSTR(), index))
            } else {
                unsafe { LoadIconW(None, IDI_APPLICATION).unwrap() }
            }
        } else { HICON(0)});
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hIcon: h.0,
            hWnd: hwnd,
            uCallbackMessage: WMU_TASKTRAY,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uID: ID_TASKTRAY,
            ..Default::default()
        };
        unsafe { std::ptr::copy(TASKTRAY_ICON_TEXT.as_ptr(),  &mut nid.szTip as _, TASKTRAY_ICON_TEXT.as_wide().len()) };
        unsafe { Shell_NotifyIconW(nim, &nid) }
    }

    fn init_hotkey(&mut self) {
        for v in PropertyHolder::enum_hotkey_param() {
            let mut str_hotkey = String::default();
            str_hotkey.push(PropertyHolder::conv_vmod2char(v.0).unwrap());
            str_hotkey.push(PropertyHolder::conv_vkey2char(v.1).unwrap());

            if v.0.0 != 0 && v.1 != VK_NONAME {
                let _ = unsafe { RegisterHotKey(None, ID_HOTKEY_1 + self.vec_hotkey_idx.len() as i32, v.0, v.1.0 as u32) };
                self.vec_hotkey_idx.push(str_hotkey.clone());
            }
            self.hash_hotkey_params.insert(str_hotkey, v.2);
        }
    }

    fn term_hotkey(&mut self) {
        self.vec_hotkey_idx.iter().enumerate().for_each(|(idx, _)| {
            let _ = unsafe { UnregisterHotKey(None, ID_HOTKEY_1 + idx as i32) };
        });
        self.vec_hotkey_idx.clear();
        self.hash_hotkey_params.clear();
    }

    fn kick_hotkey(&mut self, str_hotkey: &str) {
        let v = self.hash_hotkey_params.get(str_hotkey);
        if v.is_none() { return }

        let p = PropertyHolder::parse_string(v.unwrap());
        self.set_view_wnd(None);
        match p.hotkey_type {
            HotkeyType::WinTaskList => {
                self.set_view_wnd(Some(Box::new(WindowViewWnd::init(self.app.clone(), p.w, p.h))));
            }
            _ => { // Icon/List Launcher
                let r = ObjectHolder::bind(&p.path, p.b_sysfile_hidden);
                if r.is_ok() && r.as_ref().unwrap().list_items.len() > 0 {
                    self.set_view_wnd(Some(Box::new(FileViewWnd::init(self.app.clone(), p.hotkey_type == HotkeyType::IconLauncher,
                        p.b_icon_large, p.w as i32, p.h as i32, r.unwrap(), None))));
                }
            }
        }
        self.b_last_auto_window = false;
    }

    fn kick_arg_default(&mut self) {
        let mut str_hotkey = String::default();
        str_hotkey.push(PropertyHolder::conv_vmod2char(HOT_KEY_MODIFIERS(0)).unwrap());
        str_hotkey.push(PropertyHolder::conv_vkey2char(VK_NONAME).unwrap());

        if !self.hash_hotkey_params.contains_key(&str_hotkey) {
            unsafe { MessageBoxW(self.handle, TASKTARY_DEFAULT_CAPTION, None, MB_OK) };
        } else {
            self.kick_hotkey(&str_hotkey);
        }
    }

    fn tasktray_popup_menu(&mut self, x: i32, y: i32) {
        let r = unsafe { CreatePopupMenu() };
        let hmenu = if r.is_err() { return } else { Menu(r.unwrap()) }; // auto drop resource

        let mut mii = MENUITEMINFOW::default();
        mii.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
        mii.fMask = MIIM_ID | MIIM_STRING;
        mii.fType = MFT_STRING;

        for (idx, text) in TASKTRAY_MENU.iter().enumerate() {
            mii.wID = (idx + 1) as u32; // ゼロはポップアップメニューのキャンセル
            mii.dwTypeData = PWSTR(text.as_ptr() as _);
            let _ = unsafe { InsertMenuItemW(hmenu.0, mii.wID - 1, TRUE, &mii) };
        }

        unsafe { SetForegroundWindow(self.handle); }
        let cmd = unsafe { TrackPopupMenu(hmenu.0, TPM_LEFTALIGN | TPM_RETURNCMD, x, y, 0, self.handle, None).0 as i32 };
        let _ = unsafe { PostMessageW(self.handle, WM_NULL, WPARAM(0), LPARAM(0)) };

        match cmd {
            1 => {
                self.set_view_wnd(Some(Box::new(HotkeyPropWnd::init(self.app.clone()))));
            }
            2 => {
                self.set_view_wnd(Some(Box::new(FileViewPropWnd::init(self.app.clone(), DlgType::SortEdit, String::default(), String::default()))));
            }
            3 => {
                let _ = unsafe { DestroyWindow(self.handle) };
            }
            _ => {}
        }
    }
}

impl WndMsgHandler for MainWnd {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn set_handle(&mut self, hwnd: HWND) {
        self.handle = hwnd;
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            WM_CREATE => {
                PropertyHolder::load_winsort_param(&mut self.vec_window_sortlist);

                self.msg_taskbar_restart = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
                self.notify_icon(self.handle, NIM_ADD);
                self.init_hotkey();
            }
            WMU_TASKTRAY => {
                if wparam.0 as u32 == ID_TASKTRAY {
                    match lparam.0 as u32 {
                        WM_LBUTTONUP => {
                            self.kick_arg_default();
                        }
                        WM_RBUTTONUP => {
                            let mut pt = POINT::default();
                            let _ = unsafe { GetCursorPos(&mut pt) };
                            self.tasktray_popup_menu(pt.x, pt.y);
                        }
                        _ => { }
                    }
                } else if wparam.0 as u32 == ID_TASK_ARG { // ２重起動プロセスからトリガ
                    if lparam.0 == 0 { // 引数なしの場合
                        self.kick_arg_default();
                    } else {
                        self.kick_hotkey(&int2hotkey_str(lparam.0 as usize));
                    }
                }
            }
            WM_HOTKEY => {
                let str_hotkey = self.vec_hotkey_idx.get((wparam.0 - ID_HOTKEY_1 as usize) as usize)?;
                self.kick_hotkey(&str_hotkey.clone());
            }
            WMU_FILE_RENAME => {
                self.set_view_wnd(Some(Box::new(FileViewPropWnd::init(self.app.clone(), DlgType::Rename, self.rename_parentpath.clone(), self.rename_filename.clone()))));
                (self.rename_parentpath, self.rename_filename) = (String::default(), String::default());
            }
            WMU_DIR_PROPERTY => {
                self.set_view_wnd(Some(Box::new(FileViewPropWnd::init(self.app.clone(), DlgType::DirProperty, self.lauch_propery_dirpath.clone(), String::default()))));
                self.lauch_propery_dirpath = String::default();
            }
            WMU_HOTKEY_RELOAD => {
                self.term_hotkey();
                self.init_hotkey();
            }
            WMU_WINCLOSE => {
                self.set_view_wnd(None);
                return Some(LRESULT(0))
            }
            WM_DESTROY => {
                self.term_hotkey();
                self.notify_icon(self.handle, NIM_DELETE);
                unsafe { PostQuitMessage(0); }
            }
            _ => {
                if message == self.msg_taskbar_restart {
                    self.notify_icon(self.handle, NIM_DELETE);
                    self.notify_icon(self.handle, NIM_ADD);
                }
            }
        }
        None
    }
}

// input: 2 ascii chars, output: little endian ascii bytes(u8 * 2)
pub fn hotkey_str2u16(s: &str) -> usize {
    let buf = s.as_bytes();
    if buf.len() != 2 { return usize::MAX }
    u16::from_le_bytes([buf[0], buf[1]]) as usize
}

// input: little endian ascii bytes(u8 * 2), output: 2 ascii chars
fn int2hotkey_str(u: usize) -> String {
    let b = u16::to_le_bytes(u as u16);
    String::from_utf16_lossy(&[b[0] as u16, b[1] as u16])
}