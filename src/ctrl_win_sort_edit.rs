use fxhash::FxHashMap;
use windows::Win32::UI::Controls::{HOVER_DEFAULT, WM_MOUSELEAVE};

use super::*;
use crate::{lib_window::WindowInfo, lib_gui_layout_container::sys_metrics};

static ONCE: Once = Once::new();

#[derive(::core::cmp::PartialEq)]
enum MouseBtnState {
    NONE, LDOWN, MDOWN, RDOWN,
}

pub struct WinSortEditCtrl {
    handle: HWND,
    hfont: HFONT,

    wnd_width: i32,
    wnd_height: i32,
    item_height: i32,
    icon_width: i32,
    icon_height: i32,

    num_item: i32,
    btn_state: MouseBtnState,
    btn_idx_hover: i32,
    btn_idx_push: i32,

    pad: i32,
    group_bar_width: i32,

    vec_candidate_list: Vec<String>,
    vec_sort_list: Vec<String>,
    map_icons: FxHashMap<String, Icon>,
}

impl Drop for WinSortEditCtrl {
    fn drop(&mut self) {
    }
}

pub type WinSortEditCtrlWeak = Weak<WinSortEditCtrl>;
pub type WinSortEditCtrlRc = Rc<WinSortEditCtrl>;

impl RcValueRef<WinSortEditCtrl> for WinSortEditCtrlRc {}

pub trait WinSortEditCtrlBehavior {
    fn get_size(&self) -> (i32 /* w */, i32 /* h */);
    fn get_sortlist(&self) -> Vec<String>;
}

impl WinSortEditCtrlBehavior for WinSortEditCtrlRc {
    fn get_size(&self) -> (i32 /* w */, i32 /* h */) {
        (self.wnd_width, self.wnd_height)
    }

    fn get_sortlist(&self) -> Vec<String> {
        self.vec_sort_list.clone()
    }
}

impl WinSortEditCtrl {
    pub fn init(hparent: HWND, hfont: HFONT, sort_list: Vec<String>, cmdid: isize) -> WinSortEditCtrlWeak {
        let wnd = Rc::new(Self {
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            hfont: hfont,

            wnd_width: 0,
            wnd_height: 0,
            item_height: 0,
            icon_width: 0,
            icon_height: 0,

            num_item: 0,
            btn_state: MouseBtnState::NONE,
            btn_idx_hover: -1,
            btn_idx_push: -1,

            pad: 3,
            group_bar_width: 4,

            vec_candidate_list: Vec::<String>::default(),
            vec_sort_list: sort_list,
            map_icons: FxHashMap::<String, Icon>::default(),
        });

        let window_class = w!("win_sort_edit_ctrl");
        ONCE.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpszClassName: window_class,
                hbrBackground: unsafe { GetSysColorBrush(BRUSH_DEFAULT_BACKGROUND) },
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap(),
                lpfnWndProc: Some(wnd_proc::<Self>),
                ..Default::default()
            };
            unsafe { RegisterClassExW(&wc) };
        });

        unsafe { CreateWindowExW(
            WS_EX_COMPOSITED, window_class, w!(""), WS_CHILD | WS_VISIBLE,
            0, 0, 0, 0, hparent, HMENU(cmdid), None, Some(&wnd as *const _ as _)) };

        Rc::downgrade(&wnd)
    }

    fn view_init(&mut self) -> Result<()> {
        let s = self;

        (s.icon_width, s.icon_height) = (sys_metrics(s.handle, SM_CXSMICON), sys_metrics(s.handle, SM_CYSMICON));

        s.windowlist_init();

        let textwidth = {
            let w1 = Self::check_item_maxsize(&mut s.vec_candidate_list, unsafe { GetDC(s.handle) }, s.hfont);
            let w2 = Self::check_item_maxsize(&mut s.vec_sort_list, unsafe { GetDC(s.handle) }, s.hfont);
            if w1 < w2 { w2 } else { w1 }
        };

        s.wnd_width = s.pad * 2 + s.group_bar_width + s.pad * 2 + s.icon_width + s.pad + textwidth + s.pad;
        s.item_height = s.pad + s.icon_height + s.pad;
        s.wnd_height = s.item_height * s.num_item;

        Ok(())
    }

    fn windowlist_init(&mut self) {
        let mut nowlist = Vec::<WindowInfo>::default();
        let _ = unsafe { EnumWindows(Some(WindowInfo::enum_window), LPARAM(&mut nowlist as *mut _ as _)) };

        let r = WindowInfo::merge_proc_list(&nowlist, &self.vec_sort_list, &mut self.vec_candidate_list);
        self.icon_init(r);

        self.num_item = ( self.vec_candidate_list.len() + self.vec_sort_list.len() ) as i32;
    }

    fn icon_init(&mut self, mut handles: FxHashMap<String, HWND>) {
        let s = self;

        for v in &s.vec_candidate_list {
            let ii = Icon::load_win_icon(HWND(0), &v);
            s.map_icons.insert(v.clone(), ii);
        }
        for v in &s.vec_sort_list {
            let h = handles.remove(v).unwrap_or_default();
            let ii = Icon::load_win_icon(h, &v);
            s.map_icons.insert(v.clone(), ii);
        }
    }

    fn check_item_maxsize(v: &Vec<String>, hdc: HDC, hfont: HFONT) -> i32 {
        let mut max_width = 0;
        let mut rc  = RECT::default();

        let objold = unsafe { SelectObject(hdc, hfont) };
        for text in v {
            unsafe {DrawTextW(hdc, &mut WSTR::from(text).0, &mut rc, DT_CALCRECT | DT_TOP | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX); }
            if max_width < rc.right { max_width = rc.right };
        }
        unsafe { SelectObject(hdc, objold) };
        max_width
    }

    fn sort_item(&mut self) {
        let s = self;

        let num_cand = s.vec_candidate_list.len() as i32;
        if s.btn_idx_hover < num_cand || s.btn_idx_push == s.btn_idx_hover { return }

        let v = s.vec_sort_list.remove(( s.btn_idx_push - num_cand ) as usize);
        s.vec_sort_list.insert(( s.btn_idx_hover - num_cand ) as usize, v);
    }

    fn change_group(&mut self) {
        let s = self;

        if s.btn_idx_push < 0 { return }

        let num_cand = s.vec_candidate_list.len() as i32;
        if s.btn_idx_push < num_cand {
            let v = s.vec_candidate_list.remove(s.btn_idx_push as usize);
            s.vec_sort_list.insert(0, v);

        } else {
            let v = s.vec_sort_list.remove(( s.btn_idx_push - num_cand ) as usize);
            s.vec_candidate_list.push(v);
        }
    }

    fn calc_pt2idx(&self, mx: i32, my: i32) -> i32 { // -1:外
        let s = self;
        if mx < 0 || my < 0 || mx >= s.wnd_width || my >= s.wnd_height { -1 } else { my / s.item_height }
    }

    fn calc_idx2rect(&self, i: i32) -> RECT {
        let s = self;
        if i < 0 { return RECT::default() }

        RECT {
            left: s.pad * 2 + s.group_bar_width + s.pad * 2,
            top: s.item_height * i,
            right: s.wnd_width,
            bottom: s.item_height * i + s.item_height,
        }
    }

    fn hover_track(&mut self) {
        let s = self;

        let mut tme = TRACKMOUSEEVENT::default();
        tme.cbSize = std::mem::size_of::<TRACKMOUSEEVENT>() as u32;
        tme.dwFlags = TME_LEAVE /*| TME_HOVER*/;
        tme.hwndTrack = s.handle;
        tme.dwHoverTime = HOVER_DEFAULT;
        let _ = unsafe { TrackMouseEvent(&mut tme) };
    }

    fn mouse_handle(&mut self, mx: i32, my: i32, btn: MouseBtnState) {
        let s = self;

        let old_idx = s.btn_idx_hover;
        let sel_idx = s.calc_pt2idx(mx, my);

        if s.btn_state == MouseBtnState::MDOWN { // 中押したままカーソル移動
            if sel_idx != s.btn_idx_push { // 初回クリック対象から外れたら無効
                s.btn_idx_push = -1;
                s.btn_idx_hover = -1;
                unsafe { InvalidateRect(s.handle, None, TRUE); }
                return
            }
        }
        if btn == MouseBtnState::MDOWN { // 中クリック初回
            s.btn_idx_push = sel_idx;
            s.btn_idx_hover = sel_idx;
            unsafe { InvalidateRect(s.handle, None, TRUE); }
            return
        }

        if old_idx != sel_idx && s.btn_state == MouseBtnState::LDOWN { // 左押したままカーソル移動
            if sel_idx < s.vec_candidate_list.len() as i32 { // candidateへはソート不可
                s.btn_idx_hover = -1;
                unsafe { InvalidateRect(s.handle, None, TRUE); }
                return
            }
        }

        if btn == MouseBtnState::LDOWN { // 左クリック初回
            if sel_idx >= s.vec_candidate_list.len() as i32 { // candidateの左クリックは無効
                s.btn_idx_push = sel_idx;
                s.btn_idx_hover = sel_idx;
            }
        }

        if old_idx != sel_idx || btn == MouseBtnState::LDOWN { // フリー移動時 or 初回左クリック時
            s.hover_track(); // カーソルアウト対策用にホバー設定
            s.btn_idx_hover = sel_idx;
            unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(old_idx)), TRUE); }
            unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.btn_idx_hover)), TRUE); }
        }
    }

    fn item_draw2(&mut self, hdc: HDC, b_candidate: bool, text: &str, idx: &mut i32, y: &mut i32) {
        let  s = self;

        if !b_candidate {
            let rc = RECT {
                left: s.pad * 2,
                top: *y + s.item_height / 2 - s.group_bar_width,
                right: s.pad * 2 + s.group_bar_width,
                bottom: *y + s.item_height / 2 + s.group_bar_width };

            unsafe {
                SetDCBrushColor(hdc, COLOR_GROUPBOX);
                SetDCPenColor(hdc, COLOR_GROUPBOX);
                Rectangle(hdc, rc.left, rc.top, rc.right, rc.bottom);
            }
        }

        if *idx == s.btn_idx_push { unsafe {
            let r = s.calc_idx2rect(*idx);
            SetDCBrushColor(hdc, COLOR_DEFAULT_CURSOR_HIGHLIGHT);
            SetDCPenColor(hdc, COLOR_DEFAULT_CURSOR_HIGHLIGHT);
            Rectangle(hdc, r.left, r.top, r.right, r.bottom);
        }}

        if s.btn_idx_push >= 0 && s.btn_idx_push != s.btn_idx_hover && *idx == s.btn_idx_hover { unsafe {
            SetDCPenColor(hdc, COLOR_DEFAULT_TEXT);
            if *idx < s.btn_idx_push {
                Rectangle(hdc, 0, *y, s.wnd_width, *y + 2);
            } else {
                Rectangle(hdc, 0, *y + s.item_height - 2, s.wnd_width, *y + s.item_height);
            }

        }} else if *idx == s.btn_idx_hover  { unsafe {
            let r = s.calc_idx2rect(*idx);
            SetDCBrushColor(hdc, COLOR_DEFAULT_CURSOR_HIGHLIGHT);
            SetDCPenColor(hdc, COLOR_HIGHLIGHT_BORDER);
            Rectangle(hdc, r.left, r.top, r.right, r.bottom);
        }}

        let ii = s.map_icons.get(text);
        if let Some(i) = ii {
            let _ = unsafe { DrawIconEx(hdc, s.pad * 2 + s.group_bar_width + s.pad * 2, *y + s.pad, i.0, s.icon_width, s.icon_height, 0, None, DI_NORMAL) };
        }
        let mut rc = RECT{ left: s.pad * 2 + s.group_bar_width + s.pad * 2 + s.icon_width + s.pad, top: *y, right: s.wnd_width - s.pad, bottom: *y + s.item_height};
        unsafe { DrawTextExW(hdc, &mut WSTR::from(text).0,  &mut rc, DT_TOP | DT_VCENTER |  DT_SINGLELINE |  DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_END_ELLIPSIS, None); }
        *y += s.item_height;
        *idx += 1;
    }

    fn item_draw(&mut self, hdc: HDC) {
        let s = self;

        let objold = unsafe { SelectObject(hdc, s.hfont) };
        let old_brs = unsafe { SelectObject(hdc, GetStockObject(DC_BRUSH)) };
        let old_pen = unsafe { SelectObject(hdc, GetStockObject(DC_PEN)) };
        unsafe { SetBkMode(hdc, TRANSPARENT);}
        // このコントロールはダイアログ内で利用されるためダークモードには対応しない。文字色はデフォルト(黒のまま)。

        let mut idx = 0;
        let mut y = 0;
        for v in &s.vec_candidate_list.clone() {
            s.item_draw2(hdc, true, v, &mut idx, &mut y);
        }
        for v in &s.vec_sort_list.clone() {
            s.item_draw2(hdc, false, v, &mut idx, &mut y);
        }

        unsafe { SelectObject(hdc, old_pen) };
        unsafe { SelectObject(hdc, old_brs) };
        unsafe { SelectObject(hdc, objold) };
    }
}

impl WndMsgHandler for WinSortEditCtrl {
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
                s.view_init().ok()?;
            }
            WM_LBUTTONUP => {
                if let MouseBtnState::LDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    if s.btn_idx_push >= 0 && s.btn_idx_hover >= 0 { s.sort_item(); }
                    (s.btn_idx_push, s.btn_idx_hover) = (-1, -1);
                    unsafe { InvalidateRect(s.handle, None, TRUE) };
                }
            }
            WM_MBUTTONUP => {
                if let MouseBtnState::MDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    if s.btn_idx_push == s.btn_idx_hover && s.btn_idx_hover >= 0 { s.change_group(); }
                    (s.btn_idx_push, s.btn_idx_hover) = (-1, -1);
                    unsafe { InvalidateRect(s.handle, None, TRUE) };
                }
            }
            WM_RBUTTONUP => {
                if let MouseBtnState::RDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    unsafe { InvalidateRect(s.handle, None, TRUE) };
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
                s.btn_idx_hover = -1;
                if old_idx != -1 {
                    s.btn_idx_push = -1;
                    unsafe { InvalidateRect(s.handle, None, TRUE); }
                }
            }
            WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                unsafe { SendMessageW(GetParent(s.handle), message, wparam, lparam); }
                return Some(LRESULT(0))
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(s.handle, &mut ps) };
                s.item_draw(hdc);
                unsafe { EndPaint(s.handle, &ps); }
            }
            WM_DESTROY => {
                return Some(LRESULT(0))
            }
            _ => { }
        }
        None
    }
}
