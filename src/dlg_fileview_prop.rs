use windows::Win32::UI::Controls::EM_SETSEL;

use super::*;
use lib_gui_layout_container::*;
use lib_property::*;
use lib_shell::*;
use ctrl_win_sort_edit::*;
use dlg_hotkey_prop::*;

static ONCE: Once = Once::new();

const IDC_DUMMY: isize = 0xffff;
const IDWC_ROOT: isize = 1;
const IDWC_H1_1: isize = 11;
const IDWC_H1_2: isize = 12;
const IDWC_H12_1: isize = 16;

const IDC_SORT_EDIT:isize = 601;

const IDC_BT_OK: isize = 1001;
const IDC_BT_CANCEL: isize = 1002;
const IDC_ED_FILENAME:isize = 1003;

const WINTITLE_PROP: PCWSTR = w!("Subfolder View Property");
const WINTITLE_RENAME: PCWSTR = w!("File Rename");
const WINTITLE_WINSORT: PCWSTR = w!("Window Task Sort Edit");

pub struct FileViewPropWnd {
    app: AppWeak,
    handle: HWND,
    hfont: Font,
    dlg_type: DlgType,
    parent_parsename: String,
    target_filename: String,
    ctrl_dir_prop: Option<DirPropertyPanelRc>,
}

pub enum DlgType { Rename, DirProperty, SortEdit }

pub type FileViewPropWndWeak= Weak<FileViewPropWnd>;
pub type FileViewPropWndRc = Rc<FileViewPropWnd>;

impl RcValueRef<FileViewPropWnd> for FileViewPropWndRc {}

impl ViewWindow for FileViewPropWndWeak {
    fn close(&self) {
        if let Some(s) = self.upgrade() {
            let _ = unsafe { DestroyWindow(s.handle) };
        }
    }
}

impl FileViewPropWnd {
    pub fn init(app: AppWeak, dlg_type: DlgType, parent_parsename: String, target_filename: String) -> FileViewPropWndWeak {
        let me = Rc::new(Self {
            app: app,
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            hfont: Font(HFONT(0)),
            dlg_type: dlg_type,
            parent_parsename: parent_parsename,
            target_filename: target_filename,
            ctrl_dir_prop: None,
        });

        let window_class = w!("fileview_property_window");
        ONCE.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpszClassName: window_class,
                hbrBackground: unsafe { GetSysColorBrush(BRUSH_DEFAULT_BACKGROUND) },
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
                lpfnWndProc: Some(wnd_proc::<Self>),
                cbWndExtra: DLGWINDOWEXTRA as i32,
                ..Default::default()
            };
            unsafe { RegisterClassExW(&wc) };
        });

        let mut pt = POINT::default();
        let _ = unsafe { GetCursorPos(&mut pt) }; // DPI取得のためカーソルのあるモニタへウィンドウを生成

        unsafe { CreateWindowExW(WINDOW_EX_STYLE::default(), window_class, w!(""), WS_OVERLAPPED | WS_SYSMENU,
            pt.x, pt.y, 0, 0, None, None, None, Some(&me as *const _ as _)) };

        Rc::downgrade(&me)
    }

    fn app(&self) -> AppRc {
        self.app.upgrade().unwrap()
    }

    fn view_init_root(&mut self) -> Rc<WindowContainer> {
        (self.hfont, _) = sys_font_init(self.handle);
        let mut rc_client = RECT::default();
        let _ = unsafe { GetClientRect(self.handle, &mut rc_client) };
        let hbr = unsafe { GetSysColorBrush(BRUSH_DEFAULT_BACKGROUND) };
        let hcr = unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap();
        WindowContainerRc::create(WS_EX_CONTROLPARENT, WS_VISIBLE, hbr, hcr,
            0, 0, rc_client.right, rc_client.bottom, self.handle, HMENU(IDWC_ROOT)).upgrade().unwrap()
    }

    fn view_adjust_size_root(&mut self, c_root: &mut Rc<WindowContainer>) {
        // 必要なクライアントサイズを決定する
        c_root.recalc_layout();
        let (w_rc, h_rc) = c_root.get_field_size();
        let (mut win_w, mut win_h) = adjust_window_rect(self.handle, w_rc, h_rc);

        // 画面サイズを取得
        let mut pt = POINT::default();
        let mut mi = MONITORINFO::default();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        unsafe {
            let _ = GetCursorPos(&mut pt);
            let h = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(h, &mut mi);
        }

        // ウィンドウサイズと比較(超過分を計算)。クライアントサイズを縮小させる(スクロール設定する)
        let (mut b_add_vscr, mut b_add_hscr) = (false, false);
        for _ in 0..2 { // 初回の計算で一方にスクロールバーが生じ、もともと収まっていた他方が影響を受け両方を表示する必要があるケースへの対応
            let (w_over, h_over) = (win_w - (mi.rcWork.right - mi.rcWork.left), win_h - (mi.rcWork.bottom - mi.rcWork.top));
            if h_over > 0 {
                win_h -= h_over;
                let w_vscrbar = if !b_add_vscr { sys_metrics(self.handle, SM_CXVSCROLL) } else { 0 };
                win_w += w_vscrbar;
                b_add_vscr = true;
            }
            if w_over > 0 {
                win_w -= w_over;
                let h_hscrbar = if !b_add_hscr { sys_metrics(self.handle, SM_CYHSCROLL) } else { 0 };
                win_h += h_hscrbar;
                b_add_hscr = true;
            }
        }

        // マウス現在位置＋ウィンドウサイズが画面サイズを超える分、表示位置をマイナス方向へオフセット
        if pt.x + win_w > mi.rcWork.right { pt.x -= pt.x + win_w - mi.rcWork.right; }
        if pt.y + win_h > mi.rcWork.bottom { pt.y -= pt.y + win_h - mi.rcWork.bottom; }

        let _ = unsafe { SetWindowPos(self.handle, None, pt.x, pt.y, win_w, win_h, SWP_NOZORDER) };
        unsafe { ShowWindow(self.handle, SW_NORMAL); }
    }

    fn view_init_property(&mut self, mut c_root: Rc<WindowContainer>) -> Result<()> {
        unsafe { SetWindowTextW(self.handle, WINTITLE_PROP) }?;

        let hfont = self.hfont.0;
        let (w, h) = text_size(self.handle, hfont, CHAR_FONT_WIDTH_MEASURE);

        // プロパティパネル
        let mut c1 = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::LEFT, HeightAuto::FIX, IDWC_H1_1);
        self.ctrl_dir_prop = Some(DirPropertyPanel::init(&mut c1, hfont, false));

        // 設定・キャンセルボタン
        let mut c2: Rc<WindowContainer> = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::RIGHT, HeightAuto::FIX, IDWC_H1_2);
        init_item_hstack(&mut c2, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut c2, hfont, w * 5, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_APPLY, WINDOW_STYLE::default() | WS_TABSTOP, IDC_BT_OK);
        init_item_hstack(&mut c2, hfont, w * 5, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_CANCEL, WINDOW_STYLE::default() | WS_TABSTOP, IDC_BT_CANCEL);
        init_item_hstack(&mut c2, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        // 初期値セット
        let v = PropertyHolder::load_filesort_param(&self.parent_parsename);
        let mut p = PropertyHolder::parse_string(if v.len() > 0 { &v[0] } else { "" }); // レジストリに値が無ければデフォルト値を得る
        p.path = self.parent_parsename.clone();
        self.ctrl_dir_prop.as_mut().unwrap().ctrl_setvalue_dir_property(&p);

        self.view_adjust_size_root(&mut c_root);
        unsafe { SetFocus(get_ctrl(c2.handle(), IDC_BT_OK)); }
        Ok(())
    }

    fn apply_property(&mut self) {
        let mut p = self.ctrl_dir_prop.as_mut().unwrap().ctrl_getvalue_dir_property(true);
        p.path = String::default();
        let _ = PropertyHolder::update_dir_param(&self.parent_parsename, p.to_string());
    }

    fn view_init_rename(&mut self, mut c_root: Rc<WindowContainer>) -> Result<()> {
        unsafe { SetWindowTextW(self.handle, WINTITLE_RENAME) }?;

        let hfont = self.hfont.0;
        let (cw, ch) = text_size(self.handle, hfont, CHAR_FONT_WIDTH_MEASURE);

        let mut ce = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H1_1);
        let mut cb = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::RIGHT, HeightAuto::FIX, IDWC_H1_2);
        // ce.set_sub_proc(Some(Box::new(wnd_instance::<Self>(self.handle)))); // コントロールのsubclassを使う場合

        // ファイル名テキストボックス
        init_item_hstack(&mut ce, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut ce, hfont, -1, -1, WidthAuto::FIX, AlignV::CENTER, "STATIC", DLG_FV_ST_RENAME, WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut ce, hfont, cw * 15, ch, WidthAuto::FIX, AlignV::CENTER, "EDIT", &self.target_filename.clone(), WS_TABSTOP, IDC_ED_FILENAME);
        init_item_hstack(&mut ce, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        // ＯＫ・キャンセルボタン
        let mut cb2 = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut cb, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H12_1);
        init_item_hstack(&mut cb2, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut cb2, hfont, cw * 5, ch * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_OK, WINDOW_STYLE(BS_NOTIFY as u32) | WS_TABSTOP, IDC_BT_OK);
        init_item_hstack(&mut cb2, hfont, cw * 5, ch * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_CANCEL, WINDOW_STYLE(BS_NOTIFY as u32) | WS_TABSTOP, IDC_BT_CANCEL);
        init_item_hstack(&mut cb2, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        self.view_adjust_size_root(&mut c_root);

        // 初期値セット
        let hedit = get_ctrl(ce.handle(), IDC_ED_FILENAME);
        unsafe { SetFocus(hedit); }
        let idx = self.target_filename.chars().rev().enumerate().find(|(_, c)| '.'.eq(c)).or(Some((usize::MAX, '.'))).unwrap().0 as isize;
        unsafe { SendMessageW(hedit,  EM_SETSEL, WPARAM(0), LPARAM(self.target_filename.chars().count() as isize - idx - 1)); }

        Ok(())
    }

    fn rename_shell_item(&mut self) {
        let newname = get_ctrl_text(self.handle, IDC_ED_FILENAME);
        let _ = ObjectHolder::set_object_name(&self.parent_parsename, &self.target_filename, &newname);
    }

    fn view_init_sort_edit(&mut self, mut c_root: Rc<WindowContainer>) -> Result<()> {
        unsafe { SetWindowTextW(self.handle, WINTITLE_WINSORT) }?;

        let hfont = self.hfont.0;
        let (cw, ch) = text_size(self.handle, hfont, CHAR_FONT_WIDTH_MEASURE);

        let mut ce = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::FILL, HeightAuto::AUTO, IDWC_H1_1);
        let mut cb = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::RIGHT, HeightAuto::FIX, IDWC_H1_2);

        // ウィンドウソート順の編集コントロール
        let sortlist = self.app().main_wnd().vec_window_sortlist.iter().map(|v| match v {
                WinSortList::WILDCARD(i) => i.clone(),
                WinSortList::IMGFILE(i) => i.clone(),
            }).collect::<Vec<_>>();
        let ctrl = WinSortEditCtrl::init(ce.handle(), hfont, sortlist, IDC_SORT_EDIT).upgrade().unwrap();
        let (w, h) = ctrl.get_size();
        ce.vstack(ctrl.handle(), w, h, 2, 0, 0, false, AlignH::FILL, HeightAuto::AUTO);

        // ＯＫ・キャンセルボタン
        let mut cb2 = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut cb, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H12_1);
        init_item_hstack(&mut cb2, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut cb2, hfont, cw * 5, ch * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_OK, WINDOW_STYLE(BS_NOTIFY as u32) | WS_TABSTOP, IDC_BT_OK);
        init_item_hstack(&mut cb2, hfont, cw * 5, ch * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_FV_BT_TEXT_CANCEL, WINDOW_STYLE(BS_NOTIFY as u32) | WS_TABSTOP, IDC_BT_CANCEL);
        init_item_hstack(&mut cb2, hfont, 0, ch * 3, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        self.view_adjust_size_root(&mut c_root);

        unsafe { SetFocus(get_ctrl(cb2.handle(), IDC_BT_OK)); }
        Ok(())
    }

    fn apply_sort_setting(&mut self) {
        let hctrl = get_ctrl(self.handle, IDC_SORT_EDIT);
        let sortedit = wnd_instance::<WinSortEditCtrl>(hctrl).upgrade().unwrap();
        let after = sortedit.get_sortlist();
        let sortlist = after.into_iter().map(|v|
            if v.contains("*") { WinSortList::WILDCARD(v) } else { WinSortList::IMGFILE(v) }).collect::<Vec<_>>();
        self.app().main_wnd().get_mut().vec_window_sortlist = sortlist;
        PropertyHolder::store_winsort_param(&self.app().main_wnd().vec_window_sortlist);
    }

    fn ctrl_resize(&self, w: i32, h: i32) -> Result<()> {
        let hcont = unsafe { GetDlgItem(self.handle, IDWC_ROOT as i32) };
        unsafe { MoveWindow(hcont, 0, 0, w, h, TRUE) }?;
        Ok(())
    }
}

// impl WindowContainerSubProc for FileViewPropWndWeak { // コントロールのsubclassを使う場合
//     fn subclassproc(&mut self, hwnd: HWND, child_id: usize, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
//         None
//     }
// }

impl WindowContainerMsgProc for FileViewPropWndWeak {
    fn msgproc(&mut self, _hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match umsg {
            WM_COMMAND => {
                self.upgrade().unwrap().get_mut().message_handler(umsg, wparam, lparam);
            }
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                let hroot = unsafe { GetAncestor(self.upgrade().unwrap().handle, GA_ROOT) };
                unsafe { SendMessageW(get_ctrl(hroot, IDWC_ROOT), umsg, wparam, lparam); }
                return Some(LRESULT(0))
            }
            _ => { }
        }
        None
    }
}

impl WndMsgHandler for FileViewPropWnd {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn set_handle(&mut self, h: HWND) {
        self.handle = h;
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            WM_NCCREATE => {
                self.app().borrow_mut().dlg_wnd = Some(self.handle);
            }
            WM_NCDESTROY => {
                self.app().borrow_mut().dlg_wnd = None;
                unsafe { DefDlgProcW(self.handle, message, wparam, lparam) };
            }
            WM_CREATE => {
                let c_root = self.view_init_root();
                let _ = if let DlgType::Rename = self.dlg_type {
                    self.view_init_rename(c_root.clone())
                } else if let DlgType::DirProperty = self.dlg_type {
                    self.view_init_property(c_root.clone())
                } else { // SortEdit
                    self.view_init_sort_edit(c_root.clone())
                };
                unsafe { DefDlgProcW(self.handle, WM_ACTIVATE, WPARAM(0), LPARAM(0)); } // inactive
                unsafe { DefDlgProcW(self.handle, WM_ACTIVATE, WPARAM(1usize << u16::BITS | 0), LPARAM(0)); } // active
                return Some(LRESULT(0))
            }
            WM_ACTIVATE | WM_SETFOCUS | WM_SHOWWINDOW | WM_SYSCOMMAND => {
                return Some(unsafe { DefDlgProcW(self.handle, message, wparam, lparam) })
            }
            WM_SIZE => {
                self.ctrl_resize(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS).ok()?;
            }
            WM_COMMAND => {
                if wparam.0 >> u16::BITS == BN_CLICKED as usize {
                    let mut id = (wparam.0 & u16::MAX as usize) as isize;
                    if id == IDOK.0 as isize { id = IDC_BT_OK; }
                    if id == IDCANCEL.0 as isize { id = IDC_BT_CANCEL; }
                    match id {
                        IDC_BT_OK => {
                            if let DlgType::Rename = self.dlg_type {
                                self.rename_shell_item();
                            } else if let DlgType::DirProperty = self.dlg_type {
                                self.apply_property();
                            } else { // SortEdit
                                self.apply_sort_setting();
                            }
                            unsafe { PostMessageW(self.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) }.ok()?;
                        }
                        IDC_BT_CANCEL => {
                            unsafe { PostMessageW(self.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) }.ok()?;
                        }
                        _ => { }
                    }
                }
                return Some(LRESULT(0))
            }
            WM_DESTROY => {
                self.app().borrow_mut().dlg_wnd = None;
            }
            _ => { }
        }
        None
    }
}
