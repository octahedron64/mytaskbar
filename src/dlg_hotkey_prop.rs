use fxhash::FxHashSet;
use windows::Win32::{System::SystemServices::{SS_PATHELLIPSIS, SS_SUNKEN, }, UI::Controls::EM_LIMITTEXT, };

use super::*;
use lib_gui_layout_container::*;
use lib_property::*;

static ONCE: Once = Once::new();

const WM_CTRL_HOTKEY_DEL: u32 = WM_USER + 1001;

const WINTITLE: PCWSTR = w!("Hotkey Property");

const IDC_DUMMY: isize = 0xffff;
const IDWC_ROOT: isize = 1;
const IDWC_HOTKEYS: isize = 2;
const IDWC_H1_1: isize = 11;
const IDWC_H1_2: isize = 12;
const IDWC_H1_3: isize = 13;
const IDWC_H1_4: isize = 14;
const IDWC_H11_1: isize = 21;
const IDWC_H11_2: isize = 22;
const IDWC_H112_1: isize = 31;
const IDWC_H112_2: isize = 32;
const IDWC_H112_3: isize = 33;

const IDWC_H2_1: isize = 101;
const IDWC_H2_2: isize = 102;
const IDWC_H21_1: isize = 103;

const IDC_ST_PATH:isize = 501;
const IDC_ST_PANEL:isize = 502;

const IDC_BT_OK: isize = 1001;
const IDC_BT_CANCEL: isize = 1002;
const IDC_ED_FILENAME: isize = 1003;
const IDC_RB_ICON: isize = 1004;
const IDC_RB_LIST: isize = 1005;
const IDC_RB_ICON_L: isize = 1006;
const IDC_RB_ICON_S: isize = 1007;
const IDC_ED_LAUNCHWIN_W: isize = 1008;
const IDC_ED_LAUNCHWIN_H: isize = 1009;
const IDC_CB_DISP_HIDDEN: isize = 1010;
const IDC_DDL_HOTKEY_KIND: isize = 1011;
const IDC_DDL_ALT_SHIFT: isize = 1012;
const IDC_BT_HOTKEY_DEL: isize = 1013;
const IDC_BT_HOTKEY_ADD: isize = 1014;
const IDC_ED_HOTKEY_CHR: isize = 1015;

pub struct HotkeyPropWnd {
    app: AppWeak,
    handle: HWND,
    hfont: Font,
    ctrl_dir_prop: Vec<HotkeyPropertyPanelRc>,
}

pub type HotkeyPropWndWeak= Weak<HotkeyPropWnd>;
pub type HotkeyPropWndRc = Rc<HotkeyPropWnd>;

impl RcValueRef<HotkeyPropWnd> for HotkeyPropWndRc {}

impl ViewWindow for HotkeyPropWndWeak {
    fn close(&self) {
        if let Some(s) = self.upgrade() {
            let _ = unsafe { DestroyWindow(s.handle) };
        }
    }
}

impl HotkeyPropWnd {
    pub fn init(app: AppWeak) -> HotkeyPropWndWeak {
        let me = Rc::new(Self {
            app: app,
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            hfont: Font(HFONT(0)),
            ctrl_dir_prop: Vec::<HotkeyPropertyPanelRc>::default(),
        });

        let window_class = w!("hotkey_property_window");
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

    fn view_adjust_size_root(&mut self, c_root: &mut Rc<WindowContainer>, b_nomove_x: bool) {
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
        if b_nomove_x {
            let mut rc = RECT::default();
            let _ = unsafe { GetWindowRect(self.handle, &mut rc) };
            pt.x = rc.left;
        }

        // ウィンドウサイズと比較(超過分を計算)。クライアントサイズを縮小させる(スクロール設定する)
        let (mut b_add_vscr, mut b_add_hscr) = (false, false);
        for _ in 0..2 { // 初回の計算で一方にスクロールバーが生じ、もともと収まっていた他方が影響を受け両方を表示する必要があるケースへの対応
            let (w_over, h_over) = (win_w - (mi.rcWork.right - mi.rcWork.left), win_h - (mi.rcWork.bottom - mi.rcWork.top));
            if h_over > 0 {
                let w_vscrbar = if !b_add_vscr { sys_metrics(self.handle, SM_CXVSCROLL) } else { 0 };
                win_h -= h_over;
                win_w += w_vscrbar;
                b_add_vscr = true;
                let wc_hotkeys = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_HOTKEYS)).upgrade().unwrap();
                let p = c_root.get_vstack_param(wc_hotkeys.handle());
                c_root.set_vstack_param(wc_hotkeys.handle(), p.0 + w_vscrbar, p.1 - h_over, p.2, p.3, p.4, p.5, p.6, p.7);
            }
            if w_over > 0 {
                let h_hscrbar = if !b_add_hscr { sys_metrics(self.handle, SM_CYHSCROLL) } else { 0 };
                win_w -= w_over;
                win_h += h_hscrbar;
                b_add_hscr = true;
                let wc_hotkeys = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_HOTKEYS)).upgrade().unwrap();
                let p = c_root.get_vstack_param(wc_hotkeys.handle());
                c_root.set_vstack_param(wc_hotkeys.handle(), p.0, p.1 + h_hscrbar, p.2, p.3, p.4, p.5, p.6, p.7);
            }
        }

        // マウス現在位置＋ウィンドウサイズが画面サイズを超える分、表示位置をマイナス方向へオフセット
        if pt.x + win_w > mi.rcWork.right { pt.x -= pt.x + win_w - mi.rcWork.right; }
        if pt.y + win_h > mi.rcWork.bottom { pt.y -= pt.y + win_h - mi.rcWork.bottom; }

        let _ = unsafe { SetWindowPos(self.handle, None, pt.x, pt.y, win_w, win_h, SWP_NOZORDER) };
        unsafe { SetForegroundWindow(self.handle); }
        unsafe { ShowWindow(self.handle, SW_NORMAL); }
    }

    fn view_init_property(&mut self, mut c_root: Rc<WindowContainer>) -> Result<()> {
        let _ = unsafe { SetWindowTextW(self.handle, WINTITLE) };

        let hfont = self.hfont.0;
        let (w, h) = text_size(self.handle, hfont, CHAR_FONT_WIDTH_MEASURE);

        // hotkey part
        let mut c_hotkeys = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::LEFT, HeightAuto::FIX, IDWC_HOTKEYS);

        //////////// レジストリを読んで対象の分だけパネルを追加し、初期値をセットする
        let hotkeys = PropertyHolder::enum_hotkey_param();
        for (hk_mod, vk, paramstr/*viewParam*/) in hotkeys {
            let param = PropertyHolder::parse_string(&paramstr);

            let c_panel = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_hotkeys, 0, 0, AlignH::FILL, HeightAuto::FIX, IDC_DUMMY);
            let mut hpp = HotkeyPropertyPanel::init(&mut c_panel.clone(), hfont);
            hpp.get_mut().set_ctrl_values(hk_mod, vk, &param);
            self.ctrl_dir_prop.push(hpp);
        }

        // bottom part
        let mut c_bottom = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_root, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H1_2);

        let mut c_b_l = init_cont_hstack(wnd_instance::<Self>(self.handle), &mut c_bottom, 0, 0, WidthAuto::FIX, AlignV::FILL, IDWC_H1_3);
        init_item_hstack(&mut c_b_l, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut c_b_l, hfont, w * 5, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_HK_BT_TEXT_ADD, WINDOW_STYLE::default(), IDC_BT_HOTKEY_ADD);
        init_item_hstack(&mut c_b_l, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        let _c_b_m = init_cont_hstack(wnd_instance::<Self>(self.handle), &mut c_bottom, 0, 0, WidthAuto::AUTO, AlignV::FILL, IDC_DUMMY);

        let mut c_b_r = init_cont_hstack(wnd_instance::<Self>(self.handle), &mut c_bottom, 0, 0, WidthAuto::FIX, AlignV::FILL, IDWC_H1_4);
        init_item_hstack(&mut c_b_r, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut c_b_r, hfont, w * 5, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_HK_BT_TEXT_APPLY, WINDOW_STYLE::default(), IDC_BT_OK);
        init_item_hstack(&mut c_b_r, hfont, w * 5, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "BUTTON", DLG_HK_BT_TEXT_CANCEL, WINDOW_STYLE::default(), IDC_BT_CANCEL);
        init_item_hstack(&mut c_b_r, hfont, 0, h * 2, WidthAuto::FIX, AlignV::CENTER, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);

        self.view_adjust_size_root(&mut c_root, false);
        unsafe { SetFocus(get_ctrl(c_b_r.handle(), IDC_BT_OK)); }
        Ok(())
    }

    fn store_hotkey_param(&mut self) -> Result<()> {
        let mut vec = Vec::default();
        let mut hs = FxHashSet::<(u32/*HOT_KEY_MODIFIERS*/, u16/*VIRTUAL_KEY*/)>::default();
        let mut all_ok = true;

        for panel in &self.ctrl_dir_prop {
            let p = panel.get_ctrl_values();
            if let Some(v) = p {
                if !hs.insert((v.0.0/*HOT_KEY_MODIFIERS*/, v.1.0/*VIRTUAL_KEY*/)) { // ホットキーだぶりチェック
                    all_ok = false;
                    break;
                }
                vec.push(v);
            } else { // ホットキー指定ミス
                all_ok = false;
                break;
            }
        }
        if !all_ok {
            unsafe { MessageBoxW(self.handle, DLG_HK_CAP_INPUT_INVALID, None, MB_OK) };
            return Err(Error::OK)
        }

        PropertyHolder::store_hotkey_param(vec)?;
        Ok(())
    }

    fn add_hotkey_panel(&mut self) {
        let mut c_hotkeys = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_HOTKEYS)).upgrade().unwrap();
        let c = init_cont_vstack(wnd_instance::<Self>(self.handle), &mut c_hotkeys, 0, 0, AlignH::FILL, HeightAuto::FIX, IDC_DUMMY);
        let mut hpp = HotkeyPropertyPanel::init(&mut c.clone(), self.hfont.0);
        hpp.get_mut().set_ctrl_values(MOD_ALT | MOD_CONTROL, VK_A, &PropertyHolder::default());
        self.ctrl_dir_prop.push(hpp);

        let mut wc = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_ROOT)).upgrade().unwrap();
        wc.update_layout();
        self.view_adjust_size_root(&mut wc, true);
    }

    fn del_hotkey_panel(&mut self, rc_hotkey_panel: &HotkeyPropertyPanelRc) {
        let (idx, _) = self.ctrl_dir_prop.iter().enumerate().find(|(_, panel)| Rc::ptr_eq(panel, rc_hotkey_panel)).unwrap();
        let p = self.ctrl_dir_prop.remove(idx);
        let mut c_hotkeys = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_HOTKEYS)).upgrade().unwrap();
        c_hotkeys.remove_child(p.hparent);

        let mut wc = wnd_instance::<WindowContainer>(get_ctrl(self.handle, IDWC_ROOT)).upgrade().unwrap();
        wc.update_layout();
        self.view_adjust_size_root(&mut wc, true);
    }

    fn ctrl_resize(&self, w: i32, h: i32) -> Result<()> {
        let hcont = unsafe { GetDlgItem(self.handle, IDWC_ROOT as i32) };
        unsafe { MoveWindow(hcont, 0, 0, w, h, TRUE) }?;
        Ok(())
    }
}

impl WindowContainerMsgProc for HotkeyPropWndWeak {
    fn msgproc(&mut self, _hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match umsg {
            WM_CTRL_HOTKEY_DEL |
            WM_COMMAND => {
                self.upgrade().unwrap().get_mut().message_handler(umsg, wparam, lparam);
            }
            _ => { }
        }
        None
    }
}

impl WndMsgHandler for HotkeyPropWnd {
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
                let _ = self.view_init_property(c_root.clone());
                unsafe { DefDlgProcW(self.handle, WM_ACTIVATE, WPARAM(0), LPARAM(0)); }
                unsafe { DefDlgProcW(self.handle, WM_ACTIVATE, WPARAM(1usize << u16::BITS | 0), LPARAM(0)); }
                return Some(LRESULT(0))
            }
            WM_ACTIVATE | WM_SETFOCUS | WM_SHOWWINDOW | WM_SYSCOMMAND => {
                return Some(unsafe { DefDlgProcW(self.handle, message, wparam, lparam) });
            }
            WM_SIZE => {
                let _  = self.ctrl_resize(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS);
            }
            WM_COMMAND => {
                if wparam.0 >> u16::BITS == BN_CLICKED as usize {
                    let mut id = (wparam.0 & u16::MAX as usize) as isize;
                    if id == IDCANCEL.0 as isize { id = IDC_BT_CANCEL; } // ENTERは敢えて無反応
                    match id as isize {
                        IDC_BT_HOTKEY_ADD => {
                            self.add_hotkey_panel();
                        }
                        IDC_BT_OK => {
                            if self.store_hotkey_param().is_ok() {
                                unsafe { PostMessageW(self.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) }.ok()?;
                                unsafe { PostMessageW(self.app().main_wnd().handle(), WMU_HOTKEY_RELOAD, WPARAM(0), LPARAM(0)) }.ok()?;
                            }
                        }
                        IDC_BT_CANCEL => {
                            unsafe { PostMessageW(self.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) }.ok()?;
                        }
                        _ => { }
                    }
                }
                return Some(LRESULT(0))
            }
            WM_CTRL_HOTKEY_DEL => {
                self.del_hotkey_panel(unsafe {&*(lparam.0 as *const HotkeyPropertyPanelRc)});
                return Some(LRESULT(0))
            }
            _ => { }
        }
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
/// ホットキー設定パネル
////////////////////////////////////////////////////////////////////////////////

struct HotkeyPropertyPanel {
    hparent: HWND,
    ctrl_dir_prop: Option<DirPropertyPanelRc>,
}

type HotkeyPropertyPanelWeak = Weak<HotkeyPropertyPanel>;
type HotkeyPropertyPanelRc = Rc<HotkeyPropertyPanel>;

impl RcValueRef<HotkeyPropertyPanel> for HotkeyPropertyPanelRc {}

impl HotkeyPropertyPanel {
    fn init(wc: &mut WindowContainerRc, hfont: HFONT) -> Rc<Self> {
        let mut self_rc = Rc::new(Self {
            hparent: wc.handle(),
            ctrl_dir_prop: None,
        });

        let (_, h) = text_size(wc.handle(), hfont, CHAR_FONT_WIDTH_MEASURE); // アルファベット一文字当たりの幅(一番幅をとる文字)

        init_item_place(wc, 0.0, 0.0, 0.0, 0.0, HWND(0), HWND(0), PlaceSet::PIXEL, PlaceSet::PIXEL,
            "STATIC", "", WINDOW_STYLE(SS_SUNKEN.0), IDC_ST_PANEL);

        let mut cv0  = init_cont_place(Rc::downgrade(&self_rc), wc, 2.0, 2.0, 0.0, 0.0,
            HWND(0), HWND(0), PlaceSet::PIXEL, PlaceSet::PIXEL, IDWC_H1_1);

        let mut cv1 = init_cont_hstack(Rc::downgrade(&self_rc), &mut cv0, 0, 0, WidthAuto::FIX, AlignV::FILL, IDWC_H2_1);

        let mut cv11 = init_cont_hstack(Rc::downgrade(&self_rc), &mut cv1, 0, 0, WidthAuto::FIX, AlignV::CENTER, IDWC_H21_1);

        // コントロール配置
        init_item_vstack(&mut cv11, hfont, -1, h * 7 / 5, AlignH::CENTER, HeightAuto::FIX, "COMBOBOX", DLG_HK_DDL_HKKIND_LENGTH, WINDOW_STYLE((CBS_HASSTRINGS | CBS_DROPDOWNLIST) as u32) | WS_TABSTOP, IDC_DDL_HOTKEY_KIND);

        let mut c_hotkey = init_cont_vstack(Rc::downgrade(&self_rc), &mut cv11, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H21_1);
        init_item_hstack(&mut c_hotkey, hfont, -1, h * 7 / 5, WidthAuto::FIX, AlignV::CENTER, "COMBOBOX", DLG_HK_CB_HKALT_LENGTH, WINDOW_STYLE((CBS_HASSTRINGS | CBS_DROPDOWNLIST) as u32) | WS_TABSTOP, IDC_DDL_ALT_SHIFT);
        init_item_hstack(&mut c_hotkey, hfont, -1, -1, WidthAuto::FIX, AlignV::CENTER, "EDIT", CHAR_FONT_WIDTH_MEASURE, WINDOW_STYLE::default() | WS_TABSTOP, IDC_ED_HOTKEY_CHR);
        unsafe { SendMessageW(get_ctrl(c_hotkey.handle(), IDC_ED_HOTKEY_CHR), EM_LIMITTEXT, WPARAM(1), LPARAM(0)); }

        init_item_vstack(&mut cv11, hfont, 0, h / 2, AlignH::CENTER, HeightAuto::FIX, "STATIC", "", WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_vstack(&mut cv11, hfont, -1, h * 7 / 5, AlignH::CENTER, HeightAuto::FIX, "BUTTON", DLG_HK_BT_TEXT_DEL, WINDOW_STYLE::default() | WS_TABSTOP, IDC_BT_HOTKEY_DEL);

        // コンボボックス(ドロップダウン)選択肢セット
        let hwnd_ddl = get_ctrl(cv11.handle(), IDC_DDL_HOTKEY_KIND);
        unsafe { SendMessageW(hwnd_ddl, CB_ADDSTRING, WPARAM(0), LPARAM(DLG_HK_DDL_HKKIND[0].as_ptr() as _)); }
        unsafe { SendMessageW(hwnd_ddl, CB_ADDSTRING, WPARAM(0), LPARAM(DLG_HK_DDL_HKKIND[1].as_ptr() as _)); }

        let hwnd_ddl = get_ctrl(c_hotkey.handle(), IDC_DDL_ALT_SHIFT);
        unsafe { SendMessageW(hwnd_ddl, CB_ADDSTRING, WPARAM(0), LPARAM(DLG_HK_DDL_MODKEY[0].as_ptr() as _)); }
        unsafe { SendMessageW(hwnd_ddl, CB_ADDSTRING, WPARAM(0), LPARAM(DLG_HK_DDL_MODKEY[1].as_ptr() as _)); }
        unsafe { SendMessageW(hwnd_ddl, CB_ADDSTRING, WPARAM(0), LPARAM(DLG_HK_DDL_MODKEY[2].as_ptr() as _)); }

        let p = cv1.get_hstack_param(cv11.handle());
        cv1.set_hstack_param(cv11.handle(), p.0, p.1, 2, p.3, p.4, p.5, p.6, p.7);

        let mut cv2 = init_cont_hstack(Rc::downgrade(&self_rc), &mut cv0, 0, 0, WidthAuto::FIX, AlignV::FILL, IDWC_H2_2);

        self_rc.get_mut().ctrl_dir_prop = Some(DirPropertyPanel::init(&mut cv2, hfont, true));
        wc.recalc_layout();
        wc.recalc_layout_stop(true);

        let hwnd_st = get_ctrl(wc.handle(), IDC_ST_PANEL);
        let p = wc.get_place_param(hwnd_st);
        wc.set_place_param(hwnd_st, p.0, p.1, 2.0 , 2.0, p.4, cv0.handle(), p.6, PlaceSet::OFFSET);

        self_rc
    }

    fn set_ctrl_values(&mut self, hk_mod: HOT_KEY_MODIFIERS, vk: VIRTUAL_KEY, param: &PropertyHolder) {
        if hk_mod == (MOD_ALT | MOD_CONTROL) {
            set_ctrl_cursel(self.hparent, IDC_DDL_ALT_SHIFT, 0);
        } else if hk_mod == (MOD_SHIFT | MOD_CONTROL) {
            set_ctrl_cursel(self.hparent, IDC_DDL_ALT_SHIFT, 1);
        } else {
            set_ctrl_cursel(self.hparent, IDC_DDL_ALT_SHIFT, 2);
        }
        set_ctrl_text(self.hparent, IDC_ED_HOTKEY_CHR, &PropertyHolder::conv_vkey2char(vk).unwrap().to_string());

        if param.hotkey_type == HotkeyType::ListLauncher || param.hotkey_type == HotkeyType::IconLauncher {
            set_ctrl_cursel(self.hparent, IDC_DDL_HOTKEY_KIND, 0);
        } else if param.hotkey_type == HotkeyType::WinTaskList {
            set_ctrl_cursel(self.hparent, IDC_DDL_HOTKEY_KIND, 1);
        }

        self.ctrl_dir_prop.as_mut().unwrap().get_mut().ctrl_setvalue_dir_property(param);
    }

    fn get_ctrl_values(&self) -> Option<(HOT_KEY_MODIFIERS, VIRTUAL_KEY, PropertyHolder)> {
        let hk_mod = if get_ctrl_cursel(self.hparent, IDC_DDL_ALT_SHIFT) == 0 {
            MOD_ALT | MOD_CONTROL
        } else if get_ctrl_cursel(self.hparent, IDC_DDL_ALT_SHIFT) == 1 {
            MOD_SHIFT | MOD_CONTROL
        } else {
            HOT_KEY_MODIFIERS(0)
        };
        let c = get_ctrl_text(self.hparent, IDC_ED_HOTKEY_CHR).chars().nth(0);
        if c.is_none() { return None }
        let vkey = PropertyHolder::conv_char2vkey(c.unwrap());
        if vkey.is_err() { return None }

        let hk_kind_sel = get_ctrl_cursel(self.hparent, IDC_DDL_HOTKEY_KIND);
        let prop = self.ctrl_dir_prop.as_ref().unwrap().ctrl_getvalue_dir_property(hk_kind_sel == 0 /*LAUNCHER*/);
        Some((hk_mod, vkey.unwrap(), prop))
    }
}

impl WindowContainerMsgProc for HotkeyPropertyPanelWeak {
    fn msgproc(&mut self, hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match umsg {
            WM_MOUSEWHEEL => {
                let hroot = unsafe { GetAncestor(self.upgrade().unwrap().hparent, GA_ROOT) };
                unsafe { SendMessageW(get_ctrl(hroot, IDWC_HOTKEYS), umsg, wparam, lparam); }
                return Some(LRESULT(0))
            }
            WM_COMMAND => {
                match (wparam.0 >> u16::BITS) as u32 {
                    BN_CLICKED => {
                        match (wparam.0 & u16::MAX as usize) as isize {
                            IDC_BT_HOTKEY_DEL => {
                                let rc = self.upgrade().unwrap();
                                unsafe { SendMessageW(rc.hparent, WM_CTRL_HOTKEY_DEL, WPARAM(0), LPARAM(&rc as *const HotkeyPropertyPanelRc as _)); }
                            }
                            _ => { }
                        }
                    }
                    CBN_SELCHANGE => {
                        match (wparam.0 & u16::MAX as usize) as isize {
                            IDC_DDL_HOTKEY_KIND => { // タスクリストかランチャーか右側のパネル表示を変更する
                                let rc = self.upgrade().unwrap();
                                let sel_type_launcher = get_ctrl_cursel(rc.hparent, IDC_DDL_HOTKEY_KIND);
                                let p = rc.ctrl_dir_prop.as_ref().unwrap().ctrl_getvalue_dir_property(sel_type_launcher == 0);
                                rc.ctrl_dir_prop.as_ref().unwrap().ctrl_setvalue_dir_property(&p);
                            }
                            IDC_DDL_ALT_SHIFT => {
                                let rc = self.upgrade().unwrap();
                                let sel_alt_shift = get_ctrl_cursel(rc.hparent, IDC_DDL_ALT_SHIFT);
                                if sel_alt_shift == 0 || sel_alt_shift == 1 { // ALTかSHIFTの時はNONAME(!)の入力不可なのでクリア
                                    let c = get_ctrl_text(rc.hparent, IDC_ED_HOTKEY_CHR).chars().nth(0)?;
                                    let vkey = PropertyHolder::conv_char2vkey(c).ok()?;
                                    if vkey == VK_NONAME {
                                        set_ctrl_text(rc.hparent, IDC_ED_HOTKEY_CHR, "");
                                    }
                                }
                            }
                            _ => { }
                        }
                    }
                    EN_CHANGE => {
                        match (wparam.0 & u16::MAX as usize) as isize {
                            IDC_ED_HOTKEY_CHR => {
                                let rc = self.upgrade().unwrap();
                                let t = get_ctrl_text(rc.hparent, IDC_ED_HOTKEY_CHR).chars().nth(0);
                                if let Some(c) = t {
                                    if c.is_ascii_lowercase() {
                                        set_ctrl_text(rc.hparent, IDC_ED_HOTKEY_CHR, &c.to_ascii_uppercase().to_string());
                                    } else { 'block: {
                                        if PropertyHolder::check_hotkey_char(&c) { // 入力可能なOEMキーか
                                            let vk = PropertyHolder::conv_char2vkey(c).unwrap();
                                            if vk != VK_NONAME || (vk == VK_NONAME && get_ctrl_cursel(rc.hparent, IDC_DDL_ALT_SHIFT) == 2) {
                                                break 'block // NONAME(!)はNONE選択時のみ入力可能
                                            }
                                        }
                                        if !c.is_ascii_uppercase() && !c.is_numeric() { // 入力不可の場合はテキストを消す
                                            set_ctrl_text(rc.hparent, IDC_ED_HOTKEY_CHR, "");
                                        }
                                    }}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            WM_PAINT => {
                let rc = self.upgrade().unwrap();
                if get_ctrl(rc.hparent, IDWC_H2_1) != hwnd  { return None }

                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
                unsafe {
                    let old_pen = SelectObject(hdc, GetStockObject(DC_PEN));
                    SetDCPenColor(hdc, COLOR_CTRL_EDGE);
                    let mut r = RECT::default();
                    let _ = GetClientRect(hwnd, &mut r);
                    MoveToEx(hdc, r.right - 1, 0, None);
                    LineTo(hdc, r.right - 1, r.bottom);
                    SelectObject(hdc, old_pen);
                }
                unsafe { EndPaint(hwnd, &ps); }
            }
            _ => { }
        }
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
/// プロパティ設定パネル(ディレクトリ表示設定ダイアログでも流用)
////////////////////////////////////////////////////////////////////////////////
pub struct DirPropertyPanel {
    hparent: HWND,
    b_path_edit: bool,
}

pub type DirPropertyPanelWeak = Weak<DirPropertyPanel>;
pub type DirPropertyPanelRc = Rc<DirPropertyPanel>;

impl RcValueRef<DirPropertyPanel> for DirPropertyPanelRc {}

impl DirPropertyPanel {
    pub fn init(wc: &mut WindowContainerRc, hfont: HFONT, b_path_edit: bool) -> Rc<Self> {
        let self_rc = Rc::new(Self {
            hparent: wc.handle(),
            b_path_edit: b_path_edit,
        });

        let (w, _) = text_size(wc.handle(), hfont, CHAR_FONT_WIDTH_MEASURE); // アルファベット一文字当たりの幅(一番幅をとる文字)
        let (ew, eh) = text_size(wc.handle(), hfont, DLG_HK_ST_PROP_SIZEEDIT_SIZE);

        let mut cv1 = init_cont_hstack(Rc::downgrade(&self_rc), wc, 0, 0, WidthAuto::FIX, AlignV::FILL, IDWC_H11_1);
        let mut cv2 = init_cont_hstack(Rc::downgrade(&self_rc), wc, 0, 0, WidthAuto::AUTO, AlignV::FILL, IDWC_H11_2);

        for cap in DLG_HK_ST_PROP_CAPTIONS {
            init_item_vstack(&mut cv1, hfont, -1, -1, AlignH::RIGHT, HeightAuto::FIX, "STATIC", cap, WINDOW_STYLE::default(), IDC_DUMMY);
        }

        if b_path_edit {
            init_item_vstack(&mut cv2, hfont, w * 15, -1, AlignH::FILL, HeightAuto::FIX, "EDIT", "", WINDOW_STYLE(ES_AUTOHSCROLL as u32) | WS_TABSTOP, IDC_ED_FILENAME);
        } else {
            init_item_vstack(&mut cv2, hfont, w * 15, -1, AlignH::FILL, HeightAuto::FIX, "STATIC", "", WINDOW_STYLE(SS_PATHELLIPSIS.0 as u32), IDC_ST_PATH);
        }

        let mut c_radio1 = init_cont_vstack(Rc::downgrade(&self_rc), &mut cv2, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H112_1);
        init_item_hstack(&mut c_radio1, hfont, w * 5, -1, WidthAuto::FIX, AlignV::FILL, "BUTTON", DLG_HK_RB_LIST_ICON[0], WS_GROUP | WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP |WS_VISIBLE, IDC_RB_LIST);
        init_item_hstack(&mut c_radio1, hfont, w * 5, -1, WidthAuto::FIX, AlignV::FILL, "BUTTON", DLG_HK_RB_LIST_ICON[1], WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP, IDC_RB_ICON);

        let mut c_radio2 = init_cont_vstack(Rc::downgrade(&self_rc), &mut cv2, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H112_2);
        init_item_hstack(&mut c_radio2, hfont, w * 5, -1, WidthAuto::FIX, AlignV::FILL, "BUTTON", DLG_HK_RB_LARGE_SMALL[0], WS_GROUP | WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP, IDC_RB_ICON_L);
        init_item_hstack(&mut c_radio2, hfont, w * 5, -1, WidthAuto::FIX, AlignV::FILL, "BUTTON", DLG_HK_RB_LARGE_SMALL[1], WINDOW_STYLE(BS_AUTORADIOBUTTON as u32) | WS_TABSTOP, IDC_RB_ICON_S);

        let mut c_edit = init_cont_vstack(Rc::downgrade(&self_rc), &mut cv2, 0, 0, AlignH::FILL, HeightAuto::FIX, IDWC_H112_3);
        init_item_hstack(&mut c_edit, hfont, -1, -1, WidthAuto::FIX, AlignV::FILL, "STATIC", DLG_HK_ST_PROP_SIZEEDIT_CAP[0], WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut c_edit, hfont, ew, eh, WidthAuto::FIX, AlignV::FILL, "EDIT", "", WINDOW_STYLE::default() | WS_TABSTOP, IDC_ED_LAUNCHWIN_W);
        init_item_hstack(&mut c_edit, hfont, -1, -1, WidthAuto::FIX, AlignV::FILL, "STATIC", DLG_HK_ST_PROP_SIZEEDIT_CAP[1], WINDOW_STYLE::default(), IDC_DUMMY);
        init_item_hstack(&mut c_edit, hfont, ew, eh, WidthAuto::FIX, AlignV::FILL, "EDIT", "", WINDOW_STYLE::default() | WS_TABSTOP, IDC_ED_LAUNCHWIN_H);
        init_item_hstack(&mut c_edit, hfont, -1, -1, WidthAuto::FIX, AlignV::FILL, "STATIC", DLG_HK_ST_PROP_SIZEEDIT_CAP[2], WINDOW_STYLE::default(), IDC_DUMMY);

        init_item_vstack(&mut cv2, hfont, 0, -1, AlignH::FILL, HeightAuto::FIX, "BUTTON", DLG_HK_CB_DISP_HIDDEN, WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP, IDC_CB_DISP_HIDDEN);

        self_rc
    }

    pub fn ctrl_setvalue_dir_property(&self, param: &PropertyHolder) {
        if param.hotkey_type == HotkeyType::WinTaskList {
            set_ctrl_enable(self.hparent, IDC_ED_FILENAME, false);
            set_ctrl_enable(self.hparent, IDC_RB_ICON, false);
            set_ctrl_enable(self.hparent, IDC_RB_LIST, false);
            set_ctrl_enable(self.hparent, IDC_RB_ICON_L, false);
            set_ctrl_enable(self.hparent, IDC_RB_ICON_S, false);
            set_ctrl_enable(self.hparent, IDC_CB_DISP_HIDDEN, false);

            set_ctrl_text(self.hparent, IDC_ED_FILENAME, DLG_HK_ST_PROP_WINTASK_LIST);

        } else {
            let pathtext = if !self.b_path_edit && param.path.len() == 0 { DLG_HK_ST_PROP_PATH_DESKTOP } else { &param.path };
            if self.b_path_edit {
                set_ctrl_enable(self.hparent, IDC_ED_FILENAME, true);
                set_ctrl_text(self.hparent, IDC_ED_FILENAME, pathtext);
            } else {
                set_ctrl_enable(self.hparent, IDC_ST_PATH, true);
                set_ctrl_text(self.hparent, IDC_ST_PATH, pathtext);
            }
            set_ctrl_enable(self.hparent, IDC_RB_ICON, true);
            set_ctrl_enable(self.hparent, IDC_RB_LIST, true);
            set_ctrl_enable(self.hparent, IDC_RB_ICON_L, true);
            set_ctrl_enable(self.hparent, IDC_RB_ICON_S, true);
            set_ctrl_enable(self.hparent, IDC_CB_DISP_HIDDEN, true);

            set_ctrl_checked(self.hparent, if param.hotkey_type == HotkeyType::IconLauncher { IDC_RB_ICON } else { IDC_RB_LIST }, true); // リスト or アイコン

            if param.hotkey_type == HotkeyType::IconLauncher {
                set_ctrl_checked(self.hparent, if param.b_icon_large { IDC_RB_ICON_L } else { IDC_RB_ICON_S }, true); // 大 or 小アイコン
            } else {
                set_ctrl_checked(self.hparent, IDC_RB_ICON_L, true); // 大アイコン
                set_ctrl_enable(self.hparent, IDC_RB_ICON_L, false);
                set_ctrl_enable(self.hparent, IDC_RB_ICON_S, false);
            }

            if !param.b_sysfile_hidden {
                set_ctrl_checked(self.hparent, IDC_CB_DISP_HIDDEN, true);
            }
        }

        set_ctrl_int(self.hparent, IDC_ED_LAUNCHWIN_W, param.w as isize);
        set_ctrl_int(self.hparent, IDC_ED_LAUNCHWIN_H, param.h as isize);
    }

    pub fn ctrl_getvalue_dir_property(&self, b_type_launcher: bool) -> PropertyHolder {
        let icon_rb_status = get_ctrl_checked(self.hparent, IDC_RB_ICON);
        let b_icon_large = get_ctrl_checked(self.hparent, IDC_RB_ICON_L);
        let path = if self.b_path_edit && is_ctrl_enable(self.hparent, IDC_ED_FILENAME) {
            get_ctrl_text(self.hparent, IDC_ED_FILENAME)
        } else { String::default() };

        let (hk_type, path) = if b_type_launcher {
            if icon_rb_status {
                (HotkeyType::IconLauncher, path)
            } else {
                (HotkeyType::ListLauncher, path)
            }
        } else {
            (HotkeyType::WinTaskList, String::default())
        };
        let w = get_ctrl_int(self.hparent, IDC_ED_LAUNCHWIN_W) as u32;
        let h = get_ctrl_int(self.hparent, IDC_ED_LAUNCHWIN_H) as u32;
        let b_sysfile_hidden = !get_ctrl_checked(self.hparent, IDC_CB_DISP_HIDDEN);

        PropertyHolder::new(hk_type, b_icon_large, w, h, b_sysfile_hidden, path)
    }
}

impl WindowContainerMsgProc for DirPropertyPanelWeak {
    fn msgproc(&mut self, _hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match umsg {
            WM_MOUSEWHEEL => {
                let hroot = unsafe { GetAncestor(self.upgrade().unwrap().hparent, GA_ROOT) };
                unsafe { SendMessageW(get_ctrl(hroot, IDWC_HOTKEYS), umsg, wparam, lparam); }
                return Some(LRESULT(0))
            }
            WM_COMMAND => {
                if wparam.0 >> u16::BITS == BN_CLICKED as usize {
                    match (wparam.0 & u16::MAX as usize) as isize {
                        IDC_RB_ICON => {
                            set_ctrl_enable(self.upgrade().unwrap().hparent, IDC_RB_ICON_L, true);
                            set_ctrl_enable(self.upgrade().unwrap().hparent, IDC_RB_ICON_S, true);
                        }
                        IDC_RB_LIST => {
                            set_ctrl_enable(self.upgrade().unwrap().hparent, IDC_RB_ICON_L, false);
                            set_ctrl_enable(self.upgrade().unwrap().hparent, IDC_RB_ICON_S, false);
                        }
                        _ => { }
                    }
                }
                if wparam.0 >> u16::BITS == EN_CHANGE as usize {
                    match (wparam.0 & u16::MAX as usize) as isize {
                        IDC_ED_FILENAME => {
                            let rc = self.upgrade().unwrap();
                            let mut txt = get_ctrl_text(rc.hparent, IDC_ED_FILENAME);
                            let len = txt.len();
                            if txt.starts_with("\"") { txt.remove(0); }
                            if txt.ends_with("\"") { txt.pop(); }
                            if txt.len() != len { set_ctrl_text(rc.hparent, IDC_ED_FILENAME, &txt); }
                        }
                        _ => {}
                    }
                }
                return Some(LRESULT(0))
            }

            _ => { }
        }
        None
    }
}
