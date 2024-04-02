#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

///// global宣言

use std::rc::{Rc, Weak};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::{*},
        UI::{
            WindowsAndMessaging::*,
            Shell::*,
            Input::KeyboardAndMouse::*,
        },
    },
};

use lib_common::*;
use wnd_main::*;

const WMU_TASKTRAY: u32 = WM_USER + 101;
const WMU_WINCLOSE: u32 = WM_USER + 102;
const WMU_FILE_RENAME: u32 = WM_USER + 103;
const WMU_DIR_PROPERTY: u32 = WM_USER + 104;
const WMU_DIR_SORT_RESET: u32 = WM_USER + 105;
const WMU_HOTKEY_RELOAD: u32 = WM_USER + 106;
const ID_TASKTRAY: u32 = 0;
const ID_TASK_ARG: u32 = 1;
const ID_HOTKEY_1: i32 = 100;

mod ctrl_win_sort_edit;
mod dlg_fileview_prop;
mod dlg_hotkey_prop;
mod lib_common;
mod lib_gui_layout_container;
mod lib_property;
mod lib_shell;
mod lib_window;
mod wnd_fileview;
mod wnd_main;
mod wnd_winview;

#[cfg(feature = "en")]
mod lib_caption_en;
#[cfg(feature = "en")]
use lib_caption_en::*;
#[cfg(feature = "jp")]
mod lib_caption_jp;
#[cfg(feature = "jp")]
use lib_caption_jp::*;

///// module内宣言
use std::{cell::RefCell, sync::Once};

use windows::Win32::{
    UI::Controls::{INITCOMMONCONTROLSEX, ICC_BAR_CLASSES, InitCommonControlsEx},
    System::Ole::{OleInitialize, OleUninitialize}
};

fn main() -> Result<()> {
    AppRc::init().run()
}

pub struct App {
    main_wnd: MainWndWeak,
    dlg_wnd: Option<HWND>,
}

pub type AppRef = RefCell<App>;
pub type AppWeak = Weak<AppRef>;
pub type AppRc = Rc<AppRef>;

pub trait AppBehavior {
    fn init() -> AppRc;
    fn main_wnd(&self) -> MainWndRc;
    fn check_previous_instance() -> Result<()>;
    fn run(&mut self) -> Result<()>;
}

impl AppBehavior for AppRc {
    fn init() -> AppRc {
        Rc::new(RefCell::new(App {
            main_wnd: MainWndWeak::default(),
            dlg_wnd: None,
        }))
    }

    fn main_wnd(&self) -> MainWndRc {
        self.borrow().main_wnd.upgrade().unwrap()
    }

    fn check_previous_instance() -> Result<()> {
        if let Ok(h) = MainWnd::check_instance() {
            let args: Vec<String> = std::env::args().into_iter().collect();
            let param =
                if args.len() > 2 { return Err(Error::OK) }
                else if args.len() == 2 { hotkey_str2u16(&args[1]) }
                else { 0 }; // 引数なしの起動

            unsafe { SetForegroundWindow(h); }
            unsafe { PostMessageW(h, WMU_TASKTRAY, WPARAM(ID_TASK_ARG as usize), LPARAM(param as isize)) }?;
            return Err(Error::OK)
        }
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        if Self::check_previous_instance().is_err() { return Ok(()) } // 2重起動時は既存プロセスへタスクトレイ左クリックのメッセージポストして終了

        unsafe {
            let mut icc = INITCOMMONCONTROLSEX::default();
            icc.dwSize = std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32;
            icc.dwICC = ICC_BAR_CLASSES;
            InitCommonControlsEx(&icc);
            OleInitialize(None)?;
        }

        self.borrow_mut().main_wnd = MainWnd::init(Rc::downgrade(&self));

        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, HWND(0), 0, 0) }.into() {
            if message.message == WM_HOTKEY {
                let handle = self.main_wnd().handle();
                unsafe { SendMessageW(handle, message.message, message.wParam, message.lParam); }
            }
            if self.borrow().dlg_wnd.is_some() {
                let hdlg = self.borrow().dlg_wnd.unwrap(); // if letで受けるとIsDialogMessageW中のコールプロシジャでself二重借用発生
                if unsafe {IsDialogMessageW(hdlg, &message)}.into() { continue; }
            }
            unsafe { TranslateMessage(&message); }
            unsafe { DispatchMessageW(&message); }
        }

        unsafe {
            OleUninitialize();
            // InitCommonControlsExのクローズは無い
        }
        Ok(())
    }
}
