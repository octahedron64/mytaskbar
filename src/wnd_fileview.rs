use windows::Win32::{
    System::{Ole::IDropTarget, SystemServices::{MK_SHIFT, SFGAO_FOLDER}, Com::IDataObject, },
    UI::Controls::*,
};

use super::*;
use crate::{lib_property::{PropertyHolder, HotkeyType}, lib_shell::*, lib_gui_layout_container::{adjust_window_rect, sys_font_init, sys_metrics}};

static ONCE: Once = Once::new();

#[derive(::core::cmp::PartialEq)]
enum MouseBtnState {
    NONE, LDOWN, MDOWN, RDOWN,
}

pub struct FileViewWnd {
    app: AppWeak,
    handle: HWND,
    hfont: Font,
    handle_tooltip: HWND,

    b_icon_mode: bool,
    b_icon_large: bool,
    btn_num_col: i32,
    btn_num_row: i32,

    icon_pad: i32,
    list_pad: i32,
    scroll_height: i32,
    triangle: i32,

    wnd_width: i32,
    wnd_height: i32,

    icon_width: i32,
    icon_height: i32,
    menu_item_height: i32,

    btn_width: i32,
    btn_height: i32,

    btn_state: MouseBtnState,
    btn_idx_hover: i32,
    btn_idx_push: i32,
    sort_idx_target: i32,
    sort_idx_hover: i32,
    scroll_idx: i32,
    scroll_sel: i32,
    scroll_num: i32,

    obj: ObjectHolder,
    idrop_target: Option<Box<MyDropTargetHolder>>,
    drop_idx: i32,

    b_file_sorted: bool,
    b_block_destroy: bool,
    child_wnd: Option<Box<dyn ViewWindow>>,
    child_idx: i32,
    child_xpos: Option<u64>, // highDWORD-parent:left, lowDWORD-parent:width, lowDWORD=0の時は左方向に子ウィンドウを展開
}

impl Drop for FileViewWnd {
    fn drop(&mut self) {
        let mut list_parse_name = Vec::<String>::with_capacity(self.obj.list_items.len());
        self.obj.list_items.iter().for_each(|i| list_parse_name.push(i.str_parse_name.to_string_null_search()));
        let _ = PropertyHolder::store_filesort_param(self.b_file_sorted, &self.obj.parse_name, &mut list_parse_name);

        if let Some(c) = &self.child_wnd {
            c.close();
        }
    }
}

pub type FileViewWndWeak = Weak<FileViewWnd>;
pub type FileViewWndRc = Rc<FileViewWnd>;

impl RcValueRef<FileViewWnd> for FileViewWndRc {}

impl ViewWindow for FileViewWndWeak {
    fn close(&self) {
        if let Some(mut s) = self.upgrade() {
            s.get_mut().b_block_destroy = true;
            let _ = unsafe { DestroyWindow(s.handle) };
        }
    }

    fn is_close_blocking(&self) -> bool {
        if let Some(s) = self.upgrade() {
            if let Some(c) = &s.child_wnd {
                c.is_close_blocking()
            } else {
                s.b_block_destroy
            }
        } else { false }
    }
}

impl FileViewWnd {
    pub fn init(app:AppWeak, b_icon_mode: bool, b_large: bool, num_col: i32, num_row: i32, obj_hld: ObjectHolder, offset: Option<u64>) -> FileViewWndWeak {
        let mut wnd = Rc::new(Self {
            app: app,
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            hfont: Font(HFONT(0)),
            handle_tooltip: HWND(0),

            b_icon_mode,
            b_icon_large: b_large,
            btn_num_col: num_col,
            btn_num_row: num_row,

            icon_pad: 6,
            list_pad: 4,
            scroll_height: 16,
            triangle: 3,

            wnd_width: 0,
            wnd_height: 0,

            icon_width: 0,
            icon_height: 0,
            menu_item_height: 0,

            btn_width: 0,
            btn_height: 0,

            btn_state: MouseBtnState::NONE,
            btn_idx_hover: -1,
            btn_idx_push: -1,
            sort_idx_target: -1,
            sort_idx_hover: -1,
            scroll_idx: 0,
            scroll_sel: -1,
            scroll_num: 0,

            obj: obj_hld,
            idrop_target: None,
            drop_idx: -1,

            b_file_sorted: false,
            b_block_destroy: false,
            child_wnd: None,
            child_idx: -1,
            child_xpos: offset,
        });
        wnd.get_mut().idrop_target = Some(MyDropTargetHolder::new(Box::new(Rc::downgrade(&wnd.clone()))));

        let window_class = w!("file_view_window");
        ONCE.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                lpszClassName: window_class,
                hbrBackground: unsafe { GetSysColorBrush(BRUSH_BACKGROUND) },
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
                lpfnWndProc: Some(wnd_proc::<Self>),
                ..Default::default()
            };
            unsafe { RegisterClassExW(&wc) };
        });

        let mut pt = POINT::default();
        let _ = unsafe { GetCursorPos(&mut pt) }; // DPI取得のためカーソルのあるモニタへウィンドウを生成

        unsafe { CreateWindowExW(WS_EX_COMPOSITED, window_class, w!("My Launcher"), WS_POPUP | WS_DLGFRAME | WS_VISIBLE,
            pt.x, pt.y, 0, 0, wnd.app().main_wnd().handle(), None, None, Some(&wnd as *const _ as _)) };

        Rc::downgrade(&wnd)
    }

    fn app(&self) -> AppRc {
        self.app.upgrade().unwrap()
    }

    fn view_init(&mut self) -> Result<()> {
        let s = self;

        let num_icon = s.obj.list_items.len() as i32;

        // 画面サイズ取得
        let mut pt = POINT::default();
        let mut mi = MONITORINFO::default();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        unsafe {
            GetCursorPos(&mut pt)?;
            let h = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(h, &mut mi);
            pt.x += 1; // ダブルクリック時にフォルダを開けるように子ウィンドウの表示をずらしておく
            pt.y += 1;
        }

        if s.b_icon_mode {
            s.btn_width = sys_metrics(s.handle, if s.b_icon_large { SM_CXICON } else { SM_CXSMICON });
            s.btn_height = sys_metrics(s.handle, if s.b_icon_large { SM_CYICON } else { SM_CYSMICON });

            if s.btn_num_col == 0 && s.btn_num_row == 0 { // 縦横ともゼロの時は正方形に近づける（余りは横長）
                let mut n = 1i32;
                loop {
                    if n * n >= num_icon { break; }
                    n += 1;
                }
                if n * (n - 1) < num_icon {
                    (s.btn_num_col, s.btn_num_row) = (n, n);
                } else {
                    (s.btn_num_col, s.btn_num_row) = (n, n - 1);
                }

            } else if s.btn_num_col == 0 { // 横自動（縦を明示指定）
                s.btn_num_col = num_icon / s.btn_num_row;
                // if num_icon % s.btn_num_row != 0 { s.btn_num_col += 1; }
                // 余分を列の増でカバーすると、結果的に縦の指定数を壊すことになるのでスクロールさせる

            } else if s.btn_num_row == 0 { // 縦自動（横を明示指定）
                s.btn_num_row = num_icon / s.btn_num_col;
                if num_icon % s.btn_num_col != 0 { s.btn_num_row += 1; }
            }

            // 画面のはみ出しも含めて一旦ウィンドウサイズ計算
            s.wnd_width = (s.btn_width + s.icon_pad * 2) * s.btn_num_col;
            s.wnd_height = (s.btn_height + s.icon_pad * 2) * s.btn_num_row + s.scroll_height;

            // 画面サイズに合わせて縦・横を減算し、ウィンドウサイズを再計算
            if s.wnd_width > mi.rcWork.right - mi.rcWork.left {
                let wsize = s.btn_width + s.icon_pad * 2;
                let over = s.wnd_width - (mi.rcWork.right - mi.rcWork.left);
                s.btn_num_col -= over / wsize;
                if over % wsize > 0 { s.btn_num_col -= 1; }
                s.wnd_width = (s.btn_width + s.icon_pad * 2) * s.btn_num_col;
            }

            if s.wnd_height > mi.rcWork.bottom - mi.rcWork.top {
                let hsize = s.btn_height + s.icon_pad * 2;
                let over = s.wnd_height - (mi.rcWork.bottom - mi.rcWork.top);
                s.btn_num_row -= over / hsize;
                if over % hsize > 0 { s.btn_num_row -= 1; }
                s.wnd_height = (s.btn_height + s.icon_pad * 2) * s.btn_num_row + s.scroll_height;
            }
            s.scroll_num = s.btn_num_col * s.btn_num_row;

        } else {
            (s.hfont, _) = sys_font_init(s.handle);
            let hdc = unsafe { GetDC(s.handle)};
            let menu_itemsize = s.check_menuitem_size(hdc, s.hfont.0);

            s.icon_width = sys_metrics(s.handle, SM_CXSMICON);
            s.icon_height = sys_metrics(s.handle, SM_CYSMICON);

            (s.wnd_width, s.wnd_height) = (s.btn_num_col, s.btn_num_row);
            if s.btn_num_col == 0 || mi.rcWork.right - mi.rcWork.left < s.wnd_width  { s.wnd_width = mi.rcWork.right - mi.rcWork.left; }
            if s.btn_num_row == 0 || mi.rcWork.bottom - mi.rcWork.top < s.wnd_height { s.wnd_height = mi.rcWork.bottom - mi.rcWork.top; }

            let item_width = s.icon_width + s.list_pad * 2 + menu_itemsize + s.list_pad * 2;
            s.menu_item_height = s.icon_height + s.list_pad;

            if item_width < s.wnd_width { s.wnd_width = item_width; }

            if s.menu_item_height * num_icon + s.scroll_height < s.wnd_height {
                s.btn_num_row = num_icon;
            } else {
                let over = (s.menu_item_height * num_icon + s.scroll_height) - s.wnd_height;
                let mut num_btn_row = num_icon;
                num_btn_row -= over / s.menu_item_height;
                if over % s.menu_item_height > 0 { num_btn_row -= 1; }
                s.btn_num_row = num_btn_row;
            }
            s.wnd_height = s.menu_item_height * s.btn_num_row + s.scroll_height;
            s.scroll_num = s.btn_num_row;
        }

        unsafe { SetForegroundWindow(s.handle); }
        let (win_w, win_h) = adjust_window_rect(s.handle, s.wnd_width, s.wnd_height);

        // ウィンドウ位置の決定
        if let Some(parentpos) = s.child_xpos { // 親がいる場合は親ウィンドウの位置から決定
            let (left, width) = ((parentpos >> 32) as i32, (parentpos & u32::MAX as u64) as i32);
            if width != 0 { // 右方向へ展開
                pt.x = left + width;
                if pt.x + win_w > mi.rcWork.right { // 左方向へ転換
                    pt.x = left - s.wnd_width;
                    if pt.x < mi.rcWork.left { pt.x = mi.rcWork.left; s.child_xpos = None; } // 右方向を継続
                } else { s.child_xpos = None; } // 右方向を継続
            } else { // 左方向へ展開
                pt.x = left - s.wnd_width;
                if pt.x < mi.rcWork.left { pt.x = mi.rcWork.left; s.child_xpos = None; } // 右向きに転換
            }
        } else { // 親がないのでマウス位置から決定
            if pt.x + win_w > mi.rcWork.right { pt.x -= pt.x + win_w - mi.rcWork.right; }
        }
        // y座標は常にマウス位置から決定
        if pt.y + win_h > mi.rcWork.bottom { pt.y -= pt.y + win_h - mi.rcWork.bottom; }

        unsafe { SetWindowPos(s.handle, None, pt.x, pt.y, win_w, win_h, SWP_NOZORDER) }?;

        if s.b_icon_mode {
            return s.tooltip_init()
        }
        Ok(())
    }

    fn tooltip_init(&mut self) -> Result<()> {
        let s = self;

        let htt = unsafe { CreateWindowExW(
            WINDOW_EX_STYLE::default()/*WS_EX_TOPMOST*/, TOOLTIPS_CLASSW, None, WINDOW_STYLE(TTS_NOPREFIX  | TTS_ALWAYSTIP),
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, s.handle, None, None, None) };

        unsafe { SetWindowPos(htt, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) }?;

        s.handle_tooltip = htt;
        s.tooltip_toolset();
        unsafe { SendMessageW(htt, TTM_ACTIVATE, WPARAM(TRUE.0 as usize), LPARAM(0)) };
        Ok(())
    }

    fn tooltip_toolset(&mut self) {
        let s = self;
        let idx_last = (s.obj.list_items.len() - 1) as i32;

        let mut ti = TTTOOLINFOW {
            cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
            uFlags: TTF_SUBCLASS,
            hwnd: s.handle,
            hinst: HINSTANCE::default(),
            ..Default::default()
        };

        let mut idx = 0;
        loop {
            if idx > s.btn_num_col * s.btn_num_row - 1 { break; }
            ti.uId = idx as usize;
            unsafe { SendMessageW(s.handle_tooltip, TTM_DELTOOLW, WPARAM(0), LPARAM(&ti as *const _ as _)) };
            idx += 1;
        }

        let mut idx = 0;
        loop {
            if idx > s.btn_num_col * s.btn_num_row - 1 { break; }
            if s.scroll_idx + idx > idx_last { break; }

            ti.uId = idx as usize;
            ti.rect = s.calc_idx2rect(idx);
            ti.lpszText = s.obj.list_items[(s.scroll_idx + idx) as usize].str_disp_name.PWSTR();
            unsafe { SendMessageW(s.handle_tooltip, TTM_ADDTOOLW, WPARAM(0), LPARAM(&ti as *const _ as _)) };
            idx += 1;
        }
    }

    fn item_handle(&mut self, mut pt: POINT, b_extend: bool, b_popup: bool, b_dblclk: bool) -> Result<()> {
        let s = self;

        unsafe {ClientToScreen(s.handle, &mut pt); }

        // Drag発動の抑止
        s.btn_idx_push = -1;

        if s.btn_idx_hover >= 0 && !b_popup && s.obj.list_items[s.btn_idx_hover as usize].attr & SFGAO_FOLDER.0 != 0 {
            if b_dblclk {
                s.obj.do_menu(s.btn_idx_hover, s.app().main_wnd().handle(), pt.x, pt.y, false, false)?;
            } else if !s.b_block_destroy {
                let r = s.obj.child(s.btn_idx_hover as usize);
                if r.is_ok() && r.as_ref().unwrap().list_items.len() > 0 {
                    let p = PropertyHolder::parse_string(&r.as_ref().unwrap().str_param);
                    let mut rc = RECT::default();
                    unsafe { GetWindowRect(s.handle, &mut rc) }?;
                    let child_xpos = if s.child_xpos.is_some() { (rc.left as u64) << 32 } else { (rc.left as u64) << 32 | (rc.right- rc.left) as u64 };
                    s.b_block_destroy = true; // 子ウィンドウでドラッグ操作中にウィンドウが閉じてしまわないようブロック
                    s.child_wnd = Some(Box::new(FileViewWnd::init(s.app.clone(), p.hotkey_type == HotkeyType::IconLauncher,
                        p.b_icon_large, p.w as i32, p.h as i32, r.unwrap(), Some(child_xpos))));
                    s.child_idx = s.btn_idx_hover; // ホバーでこれ以外に当たると子ウィンドウ閉じる
                }
            }
            return Err(Error::OK)
        }
        if s.btn_idx_hover <= -2 && !b_popup { // 親フォルダへの左クリックは何もしない
            return Err(Error::OK)
        }

        // 負のidxの場合は親フォルダ自身への操作
        let r = s.obj.do_menu(s.btn_idx_hover, s.app().main_wnd().handle(), pt.x, pt.y, b_extend, b_popup);
        if r.is_err() {
            let errcode = r.clone().err().unwrap().code();

            if s.btn_idx_hover <= 0 && errcode == HRESULT(WMU_DIR_PROPERTY as i32) {
                s.app().main_wnd().get_mut().lauch_propery_dirpath = s.obj.parse_name.clone();
                unsafe { PostMessageW(s.app().main_wnd().handle(), WMU_DIR_PROPERTY, WPARAM(0), LPARAM(0)) }?;
                return r
            }
            if s.btn_idx_hover <= 0 && errcode == HRESULT(WMU_DIR_SORT_RESET as i32) {
                PropertyHolder::sort_reset(&s.obj.parse_name)?;
                return Ok(())
            }

            if s.btn_idx_hover >= 0 && errcode == HRESULT(WMU_FILE_RENAME as i32) {
                s.app().main_wnd().get_mut().rename_parentpath = s.obj.parse_name.clone();
                s.app().main_wnd().get_mut().rename_filename = s.obj.list_items[s.btn_idx_hover as usize].str_parse_name.to_string_null_search();
                unsafe { PostMessageW(s.app().main_wnd().handle(), WMU_FILE_RENAME, WPARAM(0), LPARAM(0)) }?;
                return r
            }
        }
        r
    }

    fn item_drag(&mut self) {
        let s = self;

        let r = s.obj.get_ui_object_of::<IDataObject>(s.handle, s.btn_idx_push);
        if let Ok(dobj) = r {
            s.b_block_destroy = true; // 自身がドロップ先になるケースでウィンドウが閉じてしまうことを抑止
            let _ = unsafe { ReleaseCapture() };
            s.hover_cancel();
            s.btn_state = MouseBtnState::NONE;
            let r = MyDropSource::drag_and_drop(dobj);
            s.b_block_destroy = false;
            if r.is_ok() && r.unwrap() == DRAGDROP_S_DROP {
                let _ = unsafe { PostMessageW(s.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
            }
        }
    }

    fn scroll_do(&mut self, delta: i32) {
        let s = self;
        let old = s.scroll_idx;
        s.scroll_idx -= delta;
        if s.scroll_idx + s.scroll_num > s.obj.list_items.len() as i32 {
            s.scroll_idx = s.obj.list_items.len() as i32 - s.scroll_num;
        }
        if s.scroll_idx < 0 {
            s.scroll_idx = 0;
        }
        if old != s.scroll_idx {
            if s.b_icon_mode { s.tooltip_toolset(); }
            unsafe { InvalidateRect(s.handle, None, TRUE); }
        }
    }

    fn scroll_chk(&mut self, mx: i32, my: i32, b_hover: bool) {
        let s = self;

        if !b_hover || (b_hover && s.sort_idx_target >= 0) {
            let check = s.calc_pt2idx(mx, my);
            let delta = match check {
                -3 => { s.scroll_num }
                -4 => { -s.scroll_num }
                _ => { return; }
            };
            s.scroll_sel = -1;
            s.btn_idx_push = -1;
            s.scroll_do(delta);


            if b_hover { s.hover_track(); }
        }
    }

    fn hover_track(&mut self) {
        let s = self;

        let mut tme = TRACKMOUSEEVENT::default();
        tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
        tme.dwFlags = TME_LEAVE | TME_HOVER;
        tme.hwndTrack = s.handle;
        tme.dwHoverTime = HOVER_DEFAULT;
        let _ = unsafe { TrackMouseEvent(&mut tme) };
    }

    fn hover_cancel (&mut self) {
        let s = self;

        let mut tme = TRACKMOUSEEVENT::default();
        tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
        tme.dwFlags = TME_LEAVE | TME_HOVER | TME_CANCEL;
        tme.hwndTrack = s.handle;
        let _ = unsafe { TrackMouseEvent(&mut tme) };
    }

    fn calc_idx2rect(&self, i: i32) -> RECT { // スクロール分は取り除いたidxを指定する
        let s = self;
        if i < 0 { return RECT { left: 0, top: 0, right: 0, bottom: 0 } }

        if s.b_icon_mode {
            let x = i % s.btn_num_col;
            let y = i / s.btn_num_col;
            let (w, h) = (s.btn_width + s.icon_pad * 2, s.btn_height + s.icon_pad * 2);
            RECT {
                left: x * w,
                top: y * h,
                right: x * w + w,
                bottom: y * h + h,
            }
        } else {
            RECT {
                left: 0,
                top: s.menu_item_height * i,
                right: s.wnd_width,
                bottom: s.menu_item_height * i + s.menu_item_height,
            }
        }
    }

    fn calc_pt2idx(&self, mx: i32, my: i32) -> i32 { // -1:外、-2:last_idxを超えている内側、-3:スクロール左側(上)、-4:スクロール右側(下)
        let s = self;

        if mx < 0 || my < 0 || mx > s.wnd_width || my > s.wnd_height {
            return -1
        }

        let idx_last = (s.obj.list_items.len() - 1) as i32;
        let (btw, bth) = if s.b_icon_mode {
            (s.btn_width + s.icon_pad * 2, s.btn_height + s.icon_pad * 2)
        } else {
            (s.wnd_width, s.menu_item_height)
        };

        let mut ret_idx = -1i32;
        if my < s.btn_num_row * bth {
            let idx = if s.b_icon_mode {
                mx / btw + (my / bth) * s.btn_num_col
            } else {
                my / bth
            };
            ret_idx = s.scroll_idx + idx; // スクロール計算込み
            if ret_idx > idx_last {
                return -2
            }
        } else if my >= s.btn_num_row * bth && mx <= s.wnd_width / 2 && my <= s.wnd_height {
            return -3
        } else if my >= s.btn_num_row * bth && mx > s.wnd_width / 2 && my <= s.wnd_height {
            return -4
        }
        ret_idx
    }

    fn mouse_handle(&mut self, mx: i32, my: i32, btn: MouseBtnState) {
        let s = self;

        let idx = s.calc_pt2idx(mx, my);

        if btn == MouseBtnState::MDOWN { // 中クリック初回。ソート実行中へ移行
            s.sort_idx_target = idx;
            unsafe { InvalidateRect(s.handle, None, TRUE); }
            return;
        }

        if s.sort_idx_target >= 0 { // ソート実行中
            let old_idx = s.sort_idx_hover;
            s.sort_idx_hover = idx;
            if old_idx != s.sort_idx_hover {
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(old_idx - s.scroll_idx)), TRUE); }
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.sort_idx_hover - s.scroll_idx)), TRUE); }
            }

        } else { // アイコン選択
            let old_idx = s.btn_idx_hover;
            s.btn_idx_hover = idx;
            if btn == MouseBtnState::LDOWN || btn == MouseBtnState::RDOWN { // 左・右クリック初回
                s.btn_idx_push = s.btn_idx_hover;
                s.hover_track();
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(old_idx - s.scroll_idx)), TRUE); }
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.btn_idx_hover - s.scroll_idx)), TRUE); }

            } else if old_idx != s.btn_idx_hover { // 移動中
                if s.btn_idx_push >= 0 && (s.btn_state == MouseBtnState::LDOWN || s.btn_state == MouseBtnState::RDOWN) { // 左・右押しながら移動中
                    let _ = unsafe { ReleaseCapture()};
                    s.item_drag();
                    s.btn_idx_push = -1;
                }
                s.hover_track();
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(old_idx - s.scroll_idx)), TRUE); }
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.btn_idx_hover - s.scroll_idx)), TRUE); }
            }
        }

        // ソート実行中であってもなくてもスクロールはチェック
        let old_scroll = s.scroll_sel;
        s.scroll_sel =
            if idx == -3 && s.scroll_idx > 0  { -3 }
            else if idx == -4 && s.scroll_idx + s.scroll_num < s.obj.list_items.len() as i32 { -4 }
            else { -1 };

        if old_scroll != s.scroll_sel {
            s.hover_track();
            unsafe { InvalidateRect(s.handle, None, TRUE); }
        }
    }

    pub fn check_menuitem_size(&mut self, hdc: HDC, hfont: HFONT) -> i32 {
        let mut max_width = 0;
        let mut rc: RECT = Default::default();

        let objold = unsafe { SelectObject(hdc, hfont) };
        for i in self.obj.list_items.iter_mut() {
            unsafe {DrawTextW(hdc, &mut i.str_disp_name.0, &mut rc,  DT_CALCRECT | DT_TOP | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX); }
            if max_width < rc.right { max_width = rc.right };
        }
        unsafe { SelectObject(hdc, objold) };
        max_width
    }

    fn item_draw(&mut self, hdc: HDC) {
        let s = self;

        let (w, h, pad) = if s.b_icon_mode {
            (s.btn_width, s.btn_height, s.icon_pad)
        } else {
            (s.icon_width, s.icon_height, s.list_pad)
        };

        let (old_font, old_brs_clr, old_brs, old_pen) = unsafe {
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLOR_TEXT);
            (SelectObject(hdc, s.hfont.0),
            GetDCBrushColor(hdc),
            SelectObject(hdc, GetStockObject(DC_BRUSH)),
            SelectObject(hdc, GetStockObject(DC_PEN)))
        };

        let (mut x, mut y) = (0i32, 0i32);
        let mut nowidx = 0i32;
        let mut offset = 0; // 押された時のずらし
        for i in s.obj.list_items.iter_mut() {
            if nowidx < s.scroll_idx {
                nowidx += 1;
                continue;
            }

            if nowidx == s.sort_idx_target { unsafe {
                SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                SetDCPenColor(hdc, COLOR_FILESORT_BORDER);
                if s.b_icon_mode {
                    Rectangle(hdc, x + 2, y + 2, x + w + pad * 2 - 2, y + h + pad * 2 - 2);
                } else {
                    Rectangle(hdc, 2, y + 1, s.wnd_width - 2, y + s.menu_item_height - 1);
                }
            }} else if nowidx == s.btn_idx_hover { unsafe {
                SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                SetDCPenColor(hdc, COLOR_HIGHLIGHT_BORDER);
                if s.b_icon_mode {
                    Rectangle(hdc, x + 1, y + 1, x + w + pad * 2 - 1, y + h + pad * 2 - 1);
                } else {
                    Rectangle(hdc, 1, y, s.wnd_width - 1, y + s.menu_item_height);
                }
            }}

            unsafe { SetDCBrushColor(hdc, old_brs_clr); }
            if nowidx == s.btn_idx_push {
                offset = 2;
            }
            if s.b_icon_mode {
                let ih = if s.b_icon_large { &i.icon_lr } else { &i.icon_sm };
                let _ = unsafe { DrawIconEx(hdc, x + pad + offset, y + pad + offset, ih.0, w, h, 0, None, DI_NORMAL) };
            } else {
                let _ = unsafe { DrawIconEx(hdc, pad + offset, y + pad / 2 + offset, i.icon_sm.0, w, h, 0, None, DI_NORMAL) };
                let mut rc = RECT{ left: pad + w + pad + offset, top: y + pad / 2 + offset, right: s.wnd_width - pad, bottom: y + pad + offset + h};
                unsafe { DrawTextExW(hdc, &mut i.str_disp_name.0,  &mut rc, DT_TOP | DT_VCENTER |  DT_SINGLELINE |  DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_END_ELLIPSIS, None); }

                if i.attr & SFGAO_FOLDER.0 != 0 {
                    let pt = [
                        POINT{x: 0 + s.wnd_width - pad - s.triangle, y: y + s.menu_item_height / 2 - s.triangle },
                        POINT{x: 0 + s.wnd_width - pad - s.triangle, y: y + s.menu_item_height / 2 + s.triangle },
                        POINT{x: 0 + s.wnd_width - pad    , y: y + s.menu_item_height / 2 }];
                    unsafe {
                        SetDCPenColor(hdc, COLOR_TEXT);
                        SetDCBrushColor(hdc, COLOR_TEXT);
                        Polygon(hdc, &pt);
                    }
                }
            }

            if nowidx == s.sort_idx_hover { unsafe {
                SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                SetDCPenColor(hdc, COLOR_TEXT);
                if s.sort_idx_hover > s.sort_idx_target {
                    if s.b_icon_mode {
                        Rectangle(hdc, x + w + pad * 2 - 2 , y, x + w + pad * 2 - 1, y + h + pad * 2);
                    } else {
                        Rectangle(hdc, 1 , y + s.menu_item_height, s.wnd_width - 1, y + s.menu_item_height -  1);
                    }
                } else {
                    if s.b_icon_mode {
                        Rectangle(hdc, x, y, x + 1, y + h + pad * 2);
                    } else {
                        Rectangle(hdc, 1, y, s.wnd_width - 1, y + 1);
                    }
                }
            }}

            offset = 0;
            nowidx += 1;

            if s.b_icon_mode {
                x += w + pad * 2;
                if (nowidx - s.scroll_idx) % s.btn_num_col == 0 {
                    x = 0;
                    y += h + pad * 2;
                }
            } else {
                y += s.menu_item_height;
            }

            if nowidx >= s.scroll_idx + s.scroll_num { break; }
        }

        unsafe {
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brs);
            SelectObject(hdc, old_font);
        }
    }

    fn scrollbar_draw(&mut self, hdc: HDC) {
        let s = self;
        let y_base = if s.b_icon_mode {
            s.btn_num_row * (s.btn_height + s.icon_pad * 2)
        } else {
            s.btn_num_row * s.menu_item_height
        };

        unsafe {
            let old_pen = SelectObject(hdc, GetStockObject(DC_PEN));
            SetDCPenColor(hdc, COLOR_SCROLLBAR_BORDER);

            MoveToEx(hdc, 0, y_base, None);
            LineTo(hdc, s.wnd_width, y_base);

            MoveToEx(hdc, s.wnd_width / 2, y_base, None);
            LineTo(hdc, s.wnd_width / 2, y_base + s.scroll_height);

            let old_brs = SelectObject(hdc, GetStockObject(DC_BRUSH));

            let pt = [
                POINT{x: 0 + s.wnd_width / 4 - 3, y: y_base + s.scroll_height * 3 / 4},
                POINT{x: 0 + s.wnd_width / 4    , y: y_base + s.scroll_height     / 4},
                POINT{x: 0 + s.wnd_width / 4 + 3, y: y_base + s.scroll_height * 3 / 4}];

            if s.scroll_sel == -3 {
                SetDCBrushColor(hdc, COLOR_SCROLLBTN_HIGHLIGHT);
                SetDCPenColor(hdc, COLOR_SCROLLBTN_HIGHLIGHT);
            } else {
                SetDCBrushColor(hdc, if s.scroll_idx > 0 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
                SetDCPenColor(hdc, if s.scroll_idx > 0 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
            }
            Polygon(hdc, &pt);

            let pt = [
                POINT{x: 0 + s.wnd_width * 3 / 4 - 3, y: y_base + s.scroll_height     / 4},
                POINT{x: 0 + s.wnd_width * 3 / 4    , y: y_base + s.scroll_height * 3 / 4},
                POINT{x: 0 + s.wnd_width * 3 / 4 + 3, y: y_base + s.scroll_height     / 4}];

            if s.scroll_sel == -4 {
                SetDCBrushColor(hdc, COLOR_SCROLLBTN_HIGHLIGHT);
                SetDCPenColor(hdc, COLOR_SCROLLBTN_HIGHLIGHT);
            } else {
                SetDCPenColor(hdc,
                    if s.scroll_idx + s.scroll_num - 1 < s.obj.list_items.len() as i32 - 1 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
                SetDCBrushColor(hdc,
                    if s.scroll_idx + s.scroll_num - 1 < s.obj.list_items.len() as i32 - 1 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
            }
            Polygon(hdc, &pt);

            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brs);
        }
    }
}

impl WndMsgHandler for FileViewWnd {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn set_handle(&mut self, h: HWND) {
        self.handle = h;
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        let s = self;
        match message {
            WM_CREATE => {
                s.idrop_target.as_mut().unwrap().regist(s.handle).ok()?;
                s.view_init().ok()?;
            }
            WM_DRAWITEM | WM_MEASUREITEM | WM_MENUCHAR | WM_INITMENUPOPUP => {
                s.obj.do_menu_handle(message, wparam, lparam).ok()?;
            }
            WM_LBUTTONUP => {
                if let MouseBtnState::LDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.hover_cancel();
                    s.btn_state = MouseBtnState::NONE;

                    if s.btn_idx_hover == s.btn_idx_push {
                        if s.btn_idx_hover <= -2 { // スクロールボタンへのクリック
                            s.scroll_chk(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, false);

                        } else if s.btn_idx_hover != -1 {
                            // context menuのinvoke中にWM_ACTIVATEAPPが走り、処理戻り後にSelf消滅済のケースがあるため、先にハンドル取得
                            let h = s.app().main_wnd().handle();
                            if s.item_handle(POINT { x:lparam.0 as i32 & u16::MAX as i32, y:lparam.0 as i32 >> u16::BITS },
                                wparam.0 & MK_SHIFT.0 as usize != 0, false, false).is_ok() {
                                let _ = unsafe { PostMessageW(h, WMU_WINCLOSE, WPARAM(0), LPARAM(0))};
                            }
                            return Some(LRESULT(0))
                        }
                    }
                    unsafe { InvalidateRect(s.handle, None, TRUE) };
                }
            }
            WM_LBUTTONDBLCLK => {
                if s.btn_idx_hover <= -2 { // スクロールボタン上でのボタン連打はスクロール処理に反映させる
                    s.mouse_handle(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, MouseBtnState::LDOWN);
                } else if s.btn_idx_hover != -1 {
                    let h = s.app().main_wnd().handle();
                    if s.item_handle(POINT { x:lparam.0 as i32 & u16::MAX as i32, y:lparam.0 as i32 >> u16::BITS },
                        false, false, true).is_ok() {
                        let _ = unsafe { PostMessageW(h, WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
                    }
                    return Some(LRESULT(0))
                }
            }
            WM_RBUTTONUP => {
                if let MouseBtnState::RDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.hover_cancel();
                    s.btn_state = MouseBtnState::NONE;

                    if s.btn_idx_hover == s.btn_idx_push {
                        if s.btn_idx_hover != -1 {
                            // context menuのinvoke中にWM_ACTIVATEAPPが走り、処理戻り後にSelf消滅済のケースがあるため、先にハンドル取得
                            let h = s.app().main_wnd().handle();
                            if s.item_handle(POINT { x:lparam.0 as i32 & u16::MAX as i32, y:lparam.0 as i32 >> u16::BITS },
                                wparam.0 & MK_SHIFT.0 as usize != 0, true, false).is_ok() {
                                let _ = unsafe { PostMessageW(h, WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
                            }
                            return Some(LRESULT(0))
                        }
                    }
                }
            }
            WM_MBUTTONUP => {
                if let MouseBtnState::MDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    if s.sort_idx_hover >= 0 {
                        let o = s.obj.list_items.remove(s.sort_idx_target as usize);
                        s.obj.list_items.insert(s.sort_idx_hover as usize, o);
                        if s.b_icon_mode { s.tooltip_toolset(); }
                        s.b_file_sorted = true;
                    }

                    s.sort_idx_target = -1;
                    s.sort_idx_hover = -1;
                    unsafe { InvalidateRect(s.handle, None, TRUE); }
                }
            }
            WM_LBUTTONDOWN => {
                if let MouseBtnState::NONE = s.btn_state {
                    unsafe { SetCapture(s.handle); }
                    s.mouse_handle(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, MouseBtnState::LDOWN);
                    s.btn_state = MouseBtnState::LDOWN;
                }
            }
            WM_MBUTTONDOWN => {
                if let MouseBtnState::NONE = s.btn_state {
                    unsafe { SetCapture(s.handle); }
                    s.mouse_handle(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, MouseBtnState::MDOWN);
                    s.btn_state = MouseBtnState::MDOWN;
                }
            }
            WM_RBUTTONDOWN => {
                if let MouseBtnState::NONE = s.btn_state {
                    unsafe { SetCapture(s.handle); }
                    s.mouse_handle(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, MouseBtnState::RDOWN);
                    s.btn_state = MouseBtnState::RDOWN;
                }
            }
            WM_MOUSEMOVE => {
                s.mouse_handle(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, MouseBtnState::NONE);
            }
            WM_MOUSELEAVE => {
                let old_idx = s.btn_idx_hover;
                let old_scroll = s.scroll_sel;
                s.btn_idx_hover = -1;
                s.scroll_sel = -1;
                if old_idx != s.btn_idx_hover || old_scroll != s.scroll_sel {
                    s.btn_idx_push = -1;
                    unsafe { InvalidateRect(s.handle, None, TRUE); }
                }
            }
            WM_MOUSEHOVER => {
                // ソート実行中のスクロール
                s.scroll_chk(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, true);

                // 子ウィンドウ開いている時に、親のホバーで子ウィンドウを閉じる
                if let Some(_) = s.child_wnd  {
                    if s.btn_idx_hover >= 0 && s.btn_idx_hover != s.child_idx {
                        unsafe { SetForegroundWindow(s.handle); }
                    }
                }
            }
            WM_MOUSEWHEEL => {
                if s.scroll_num < s.obj.list_items.len() as i32 {
                    let mut delta = (wparam.0 >> u16::BITS) as i16 / WHEEL_DELTA as i16;
                    if s.b_icon_mode { delta *= s.btn_num_col as i16; }
                    s.scroll_do(delta as i32);
                }
                return Some(LRESULT(0))
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(s.handle, &mut ps) };

                s.item_draw(hdc);

                if s.scroll_num < s.obj.list_items.len() as i32 {
                    s.scrollbar_draw(hdc);
                }

                unsafe { EndPaint(s.handle, &ps); }
            }
            WM_ACTIVATEAPP => {
                if wparam.0 == 0 /* INACTIVATE */ && !s.b_block_destroy {
                    let _ = unsafe { PostMessageW(s.app().main_wnd().handle(), WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
                    return Some(LRESULT(0))
                }
            }
            WM_ACTIVATE => {
                if wparam.0 != 0 { // activateされようとしている場合
                    if let Some(c) = &s.child_wnd  {
                        if !c.is_close_blocking() {
                            c.close();
                            s.child_wnd = None;
                            s.b_block_destroy = false;
                            return Some(LRESULT(0))
                        }
                    }
                }
            }
            WM_DESTROY => {
                s.idrop_target.as_ref().unwrap().unregist(s.handle).ok()?;
            }
            _ => { }
        }
        None
    }
}

impl DropTargetWindow for FileViewWndWeak {
    fn get_handle(&self) -> HWND {
        self.upgrade().unwrap().handle
    }

    // Result=ErrはDropTarget維持(無しの場合も含め)を意味する
    // Result=OkのOption=Noneは、対象アイテムにDropTargetが無いことを意味する
    fn get_droptarget(&mut self, mx: i32, my: i32, b_enter: bool) -> (Result<()>, Option<IDropTarget>) {
        let mut sr = self.upgrade().unwrap();
        let mut idx = sr.calc_pt2idx(mx, my);
        if idx == -2 || idx == -3 || idx == -4 {
            idx = -2;
        }

        if !b_enter {
            if sr.drop_idx == idx {
                sr.get_mut().drop_idx = idx;
                return (Err(Error::OK), None) // 変化がないのでDropTargetは維持
            } else if idx == -1 {
                sr.get_mut().drop_idx = idx;
                return (Ok(()), None) // 変化してDropTarget無しになった
            }
        } else if idx == -1 {
            sr.get_mut().drop_idx = idx;
            return (Err(Error::OK), None) // 外なのでDropTargetは無し
        }

        let r =
            if idx == -2 || idx >= 0 {
                sr.obj.get_ui_object_of::<IDropTarget>(sr.handle, idx)
            } else {
                Err(Error::OK)
            };

        sr.get_mut().drop_idx = idx;
        if r.is_err() {
            return (Ok(()), None) // DropTarget無し
        } else {
            return (Ok(()), Some(r.unwrap())) // DropTarget有り
        }
    }
}
