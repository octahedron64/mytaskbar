use std::collections::VecDeque;
use fxhash::FxHashMap;
use windows::Win32::UI::Controls::{HOVER_DEFAULT, WM_MOUSEHOVER, WM_MOUSELEAVE};

use self::lib_property::PropertyHolder;

use super::*;
use crate::{lib_window::WindowInfo, lib_gui_layout_container::{adjust_window_rect, sys_font_init, sys_metrics}};

static ONCE: Once = Once::new();

#[derive(::core::cmp::PartialEq)]
enum MouseBtnState {
    NONE, LDOWN, MDOWN, RDOWN,
}

pub struct WindowViewWnd {
    app: AppWeak,
    handle: HWND,
    hfont: Font,
    b_block_destroy: bool,

    wnd_width: i32,
    wnd_height: i32,
    item_height: i32,
    icon_width: i32,
    icon_height: i32,

    num_item: i32,
    btn_state: MouseBtnState,
    btn_idx_hover: i32,
    btn_idx_push: i32,
    grp_idx_push: i32,
    grp_idx_sort_target: i32,
    wnd_idx_push: i32,
    wnd_idx_target: i32,
    wnd_b_target_upper: bool,
    scroll_idx: i32,
    scroll_sel: i32,
    scroll_num: i32,

    pad: i32,
    group_bar_width: i32,
    scroll_height: i32,

    vec_items: VecDeque<VecDeque<WindowInfo>>,
    map_icons: FxHashMap<isize/* HWND */, Icon>,
}

impl Drop for WindowViewWnd {
    fn drop(&mut self) {
    }
}

pub type WindowViewWndWeak = Weak<WindowViewWnd>;
pub type WindowViewWndRc = Rc<WindowViewWnd>;

impl RcValueRef<WindowViewWnd> for WindowViewWndRc {}

impl ViewWindow for WindowViewWndWeak {
    fn close(&self) {
        if let Some(mut s) = self.upgrade() {
            s.get_mut().b_block_destroy = true;
            let _ = unsafe { DestroyWindow(s.handle) };
        }
    }
}

impl WindowViewWnd {
    pub fn init(app: AppWeak, w: u32, h: u32) -> WindowViewWndWeak {
        let wnd = Rc::new(Self {
            app: app,
            handle: HWND(0), // WM_NCCREATEの処理の中で設定される
            hfont: Font(HFONT(0)),
            b_block_destroy: false,

            wnd_width: w as _,
            wnd_height: h as _,
            item_height: 0,
            icon_width: 0,
            icon_height: 0,

            num_item: 0,
            btn_state: MouseBtnState::NONE,
            btn_idx_hover: -1,
            btn_idx_push: -1,
            grp_idx_push: -1,
            grp_idx_sort_target: -1,
            wnd_idx_push: -1,
            wnd_idx_target: -1,
            wnd_b_target_upper: true,
            scroll_idx: 0,
            scroll_sel: -1,
            scroll_num: 0,

            // 2-3-2-icon-2-text
            pad: 3,
            group_bar_width: 4,
            scroll_height: 16,

            vec_items: VecDeque::<VecDeque<WindowInfo>>::default(),
            map_icons: FxHashMap::<isize/* HWND */, Icon>::default(),
        });

        let window_class = w!("window_view_window");

        ONCE.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
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
            pt.x, pt.y, 0, 0, wnd.app().main_wnd().handle(), None, None, Some(&wnd as *const _ as _)
        ) };

        Rc::downgrade(&wnd)
    }

    fn app(&self) -> AppRc {
        self.app.upgrade().unwrap()
    }

    fn view_init(&mut self) -> Result<()> {
        let s = self;

        s.icon_width = sys_metrics(s.handle, SM_CXSMICON);
        s.icon_height = sys_metrics(s.handle, SM_CYSMICON);

        s.windowlist_init();
        s.icon_init();

        (s.hfont, _) = sys_font_init(s.handle);
        let textwidth = unsafe { Self::check_item_maxsize(&mut s.vec_items, GetDC(s.handle), s.hfont.0) };

        // 画面サイズ取得
        let mut pt = POINT::default();
        let mut mi = MONITORINFO::default();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        unsafe {
            GetCursorPos(&mut pt)?;
            let h = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(h, &mut mi);
        }
        if s.wnd_width == 0 || mi.rcWork.right - mi.rcWork.left < s.wnd_width  { s.wnd_width = mi.rcWork.right - mi.rcWork.left; }
        if s.wnd_height == 0 || mi.rcWork.bottom - mi.rcWork.top < s.wnd_height { s.wnd_height = mi.rcWork.bottom - mi.rcWork.top; }

        let item_width = s.pad * 2 + s.group_bar_width + s.pad * 2 + s.icon_width + s.pad + textwidth + s.pad;
        s.item_height = s.pad + s.icon_height + s.pad;

        if item_width < s.wnd_width { s.wnd_width = item_width; }

        s.scroll_num = s.num_item;
        if s.item_height * s.num_item + s.scroll_height < s.wnd_height {
            s.wnd_height = s.item_height * s.num_item + s.scroll_height;
        } else {
            let over = (s.item_height * s.num_item + s.scroll_height) - s.wnd_height;
            s.scroll_num -= over / s.item_height;
            if over % s.item_height > 0 { s.scroll_num -= 1; }
            s.wnd_height = s.item_height * s.scroll_num + s.scroll_height;
        }

        unsafe { SetForegroundWindow(s.handle); }
        let (win_w, win_h) = adjust_window_rect(s.handle, s.wnd_width, s.wnd_height);

        // マウス位置からウィンドウ位置を計算
        if pt.x + win_w > mi.rcWork.right { pt.x -= pt.x + win_w - mi.rcWork.right; }
        if pt.y + win_h > mi.rcWork.bottom { pt.y -= pt.y + win_h - mi.rcWork.bottom; }

        unsafe { SetWindowPos(s.handle, None, pt.x, pt.y, win_w, win_h, SWP_NOZORDER) }?;

        Ok(())
    }

    fn windowlist_init(&mut self) {
        let main_wnd = self.app().main_wnd();
        self.vec_items = main_wnd.vec_window_items.clone();
        let sortlist = &main_wnd.vec_window_sortlist;

        let mut nowlist = Vec::<WindowInfo>::default();
        let _ = unsafe { EnumWindows(Some(WindowInfo::enum_window), LPARAM(&mut nowlist as *mut _ as _)) };
        self.num_item = nowlist.len() as i32;

        WindowInfo::sort_window_list(&sortlist, nowlist, &mut self.vec_items);
    }

    fn icon_init(&mut self) {
        let s = self;

        for v in &s.vec_items {
            for wi in v {
                let ic = Icon::load_win_icon(wi.handle, &wi.proc_img_fname);
                s.map_icons.insert(wi.handle.0, ic);
            }
        }
    }

    pub fn check_item_maxsize(v: &mut VecDeque<VecDeque<WindowInfo>>, hdc: HDC, hfont: HFONT) -> i32 {
        let mut max_width = 0;
        let mut rc: RECT = Default::default();

        let objold = unsafe { SelectObject(hdc, hfont) };
        for vv in v.iter_mut() {
            for i in vv.iter_mut() {
                unsafe {DrawTextW(hdc, &mut WSTR::from(&i.title).0, &mut rc, DT_CALCRECT | DT_TOP | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX); }
                if max_width < rc.right { max_width = rc.right };
            }
        }
        unsafe { SelectObject(hdc, objold) };
        max_width
    }

    fn item_handle(&mut self, mut pt: POINT, b_popup: bool) -> Result<()> {
        let s = self;

        unsafe {ClientToScreen(s.handle, &mut pt); }

        if s.btn_idx_hover >= 0 {
            let mut v_lastidx = -1i32;
            for v in &s.vec_items {
                v_lastidx += v.len() as i32;
                if v_lastidx >= s.btn_idx_hover { unsafe {
                    let idx = v.len() as i32 - 1 - (v_lastidx - s.btn_idx_hover);
                    if !b_popup {
                        if IsIconic(v[idx as usize].handle) == TRUE {
                            SendMessageTimeoutW(v[idx as usize].handle, WM_SYSCOMMAND, WPARAM(SC_RESTORE as usize), LPARAM(0), SMTO_ABORTIFHUNG | SMTO_BLOCK, 500, None);
                        }
                        SetForegroundWindow(v[idx as usize].handle);
                    } else {
                        SetWindowPos(s.handle, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)?;
                        SetForegroundWindow(v[idx as usize].handle);
                        SendMessageTimeoutW(v[idx as usize].handle, 0x313, WPARAM(0), LPARAM((pt.y << u16::BITS | pt.x) as isize), SMTO_NOTIMEOUTIFNOTHUNG | SMTO_BLOCK, 500, None);
                        // システムメニュー(最大化、閉じるなど)の表示(undocument message)
                    }
                    break;
                }}
            }
        }
        Err(Error::OK)
    }

    fn sort_group(&mut self) {
        let s = self;

        if s.grp_idx_sort_target < 0  || s.grp_idx_push == s.grp_idx_sort_target { return }

        let v = s.vec_items.remove(s.grp_idx_push as usize);
        s.vec_items.insert(s.grp_idx_sort_target as usize, v.unwrap());
    }

    fn sort_window(&mut self) {
        let s = self;

        if s.wnd_idx_target < 0 { return }

        let mut v_lastidx = -1i32;
        let (mut push_grp, mut push_idx) = (-1i32, -1i32);
        for (g, v) in s.vec_items.iter().enumerate() { // 中クリックした対象アイテムを特定
            v_lastidx += v.len() as i32;
            if v_lastidx >= s.wnd_idx_push {
                let i = v.len() as i32 - 1 - (v_lastidx - s.wnd_idx_push);
                (push_grp, push_idx) = (g as i32, i);
                break;
            }
        }

        if s.wnd_idx_push == s.wnd_idx_target { // 中クリックの処理

            // フリーグループ＆単独アイテムの中クリック：既に同じプロセスグループ(テンポラリでない)が存在したら何もしない。でなければプロセスグループ(0 or 1)に戻す。
            if s.vec_items[push_grp as usize][push_idx as usize].group_type == 2 && s.vec_items[push_grp as usize].len() == 1 {
                for (g, v) in s.vec_items.iter().enumerate() {
                    if g != push_grp as usize && (v[0].group_type == 0 || v[0].group_type == 1) &&
                        s.vec_items[push_grp as usize][push_idx as usize].proc_img_fname.eq(&v[0].proc_img_fname) {
                        return
                    }
                }

                if PropertyHolder::contains_window_sort_list(
                    &s.app().main_wnd().vec_window_sortlist,
                    &s.vec_items[push_grp as usize][push_idx as usize].proc_img_fname)
                    .is_some() {
                    s.vec_items[push_grp as usize][push_idx as usize].group_type = 0;
                } else {
                    s.vec_items[push_grp as usize][push_idx as usize].group_type = 1;
                }
                return
            }

            // それ以外の場合は対象アイテムをフリーグループへ外出し
            let mut wnd = s.vec_items[push_grp as usize].remove(push_idx as usize).unwrap();
            if s.vec_items[push_grp as usize].len() == 0 {
                s.vec_items.remove(push_grp as usize);
            }
            wnd.group_type = 2;
            let mut newgrp = VecDeque::<WindowInfo>::default();
            newgrp.push_back(wnd);
            s.vec_items.insert(push_grp as usize, newgrp);
            return
        }

        let mut v_lastidx = -1i32;
        let (mut target_grp, mut target_idx) = (-1i32, -1i32);
        for (g, v) in s.vec_items.iter().enumerate() { // ソート先の対象アイテムを特定
            v_lastidx += v.len() as i32;
            if v_lastidx >= s.wnd_idx_target {
                let mut i = v.len() as i32 - 1 - (v_lastidx - s.wnd_idx_target);
                if push_grp == g as i32 && push_idx <= i { i -= 1; }
                if !s.wnd_b_target_upper { i += 1; }
                (target_grp, target_idx) = (g as i32, i);
                break;
            }
        }

        let target_group_type = s.vec_items[target_grp as usize][0].group_type;
        // プロセスグループ(PG)に所属するアイテムは別のPGへ移動できない
        if push_grp != target_grp && s.vec_items[push_grp as usize][0].group_type != 2 && target_group_type != 2 { return }
        // フリーグループに所属するアイテムが他のPGへ移動するときはproc_imgが一致必要
        if push_grp != target_grp && s.vec_items[push_grp as usize][0].group_type == 2 && target_group_type != 2 &&
            s.vec_items[push_grp as usize][push_idx as usize].proc_img_fname.ne(&s.vec_items[target_grp as usize][0].proc_img_fname) { return }

        // 個別ウィンドウのソートを実行(グループ内 or グループまたぎ　問わず)
        let mut wnd = s.vec_items[push_grp as usize].remove(push_idx as usize).unwrap();
        wnd.group_type = target_group_type;
        s.vec_items[target_grp as usize].insert(target_idx as usize, wnd);
        if s.vec_items[push_grp as usize].len() == 0 {
            s.vec_items.remove(push_grp as usize);
        }
    }

    fn calc_selidx2grpidx(&self, selidx: i32) -> i32 {
        let s = self;
        if selidx < 0 { return selidx }

        let mut grpidx = -1i32;
        let mut v_lastidx = -1i32;
        for v in &s.vec_items {
            grpidx += 1;
            v_lastidx += v.len() as i32;
            if v_lastidx >= selidx {
                break;
            }
        }
        grpidx
    }

    fn calc_pt2idx(&self, mx: i32, my: i32) -> (i32, bool) { // -1:外、-2:last_idxを超えている内側、-3:スクロール左側(上)、-4:スクロール右側(下)
        let s = self;

        if mx < 0 || my < 0 || mx > s.wnd_width || my > s.wnd_height {
            return (-1, true)
        }

        if my < s.scroll_num * s.item_height {
            let mut idx = my / s.item_height;
            let b = (my % s.item_height) < s.item_height / 2;
            idx = s.scroll_idx + idx; // スクロール計算込み
            if idx > s.num_item - 1 {
                return (-2, true)
            }
            return (idx, b)
        } else if my >= s.scroll_num * s.item_height && mx <= s.wnd_width / 2 && my <= s.wnd_height {
            return (-3, true)
        } else if my >= s.scroll_num * s.item_height && mx > s.wnd_width / 2 && my <= s.wnd_height {
            return (-4, true)
        } else {
            return (-1, true)
        }
    }

    fn calc_idx2rect(&self, i: i32) -> RECT { // スクロール分は取り除いたidxを指定する
        let s = self;
        if i < 0 { return RECT { left: 0, top: 0, right: 0, bottom: 0 } }

        RECT {
            left: s.pad * 2 + s.group_bar_width + s.pad * 2,
            top: s.item_height * i,
            right: s.wnd_width,
            bottom: s.item_height * i + s.item_height,
        }
    }

    fn scroll_do(&mut self, delta: i32) {
        let s = self;
        let old = s.scroll_idx;
        s.scroll_idx -= delta;
        if s.scroll_idx < 0 {
            s.scroll_idx = 0;
        }
        if s.scroll_idx + s.scroll_num > s.num_item {
            s.scroll_idx = s.num_item - s.scroll_num;
        }
        if old != s.scroll_idx {
            unsafe { InvalidateRect(s.handle, None, TRUE); }
        }
    }

    fn scroll_chk(&mut self, mx: i32, my: i32, b_hover: bool) {
        let s = self;

        if !b_hover || (b_hover && (s.grp_idx_push >= 0 || s.wnd_idx_push >= 0)) {
            let (check, _) = s.calc_pt2idx(mx, my);
            let delta = match check {
                -3 => { s.scroll_num }
                -4 => { -s.scroll_num }
                _ => { return }
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

    fn mouse_handle(&mut self, mx: i32, my: i32, btn: MouseBtnState) {
        let s = self;

        let old_idx = s.btn_idx_hover;
        let (sel_idx, b_sel_upper) = s.calc_pt2idx(mx, my);

        if btn == MouseBtnState::MDOWN { // 中クリック初回
            s.wnd_idx_push = sel_idx;
            (s.wnd_idx_target, s.wnd_b_target_upper) = (sel_idx, true);
            unsafe { InvalidateRect(s.handle, None, TRUE); }
            return
        }
        if s.btn_state == MouseBtnState::MDOWN { // 中押したままカーソル移動
            let (wnd_old_idx, wnd_old_b_upper) = (s.wnd_idx_target, s.wnd_b_target_upper);
            (s.wnd_idx_target, s.wnd_b_target_upper) =  (sel_idx, b_sel_upper);

            if (wnd_old_idx, wnd_old_b_upper) != (s.wnd_idx_target, s.wnd_b_target_upper) {
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(wnd_old_idx - s.scroll_idx)), TRUE); }
                unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.wnd_idx_target - s.scroll_idx)), TRUE); }
                return
            }
        }

        if btn == MouseBtnState::LDOWN || btn == MouseBtnState::RDOWN { // 左・右クリック初回
            s.btn_idx_push = sel_idx;
            s.btn_idx_hover = sel_idx;
        }
        if old_idx != sel_idx && s.btn_state == MouseBtnState::LDOWN { // 左押したままカーソル移動
            if s.grp_idx_push == -1 { // グループソート開始
                s.grp_idx_push = s.calc_selidx2grpidx(old_idx);
                s.grp_idx_sort_target = s.grp_idx_push;
                unsafe { InvalidateRect(s.handle, None, TRUE); }
            }
            if s.grp_idx_push != -1 { // グループソート選択中
                let old = s.grp_idx_sort_target;
                s.grp_idx_sort_target = s.calc_selidx2grpidx(sel_idx);
                if old != s.grp_idx_sort_target {
                    unsafe { InvalidateRect(s.handle, None, TRUE); }
                }
            }
        }
        if old_idx != sel_idx || (btn == MouseBtnState::LDOWN || btn == MouseBtnState::RDOWN) { // クリック時or移動時
            s.hover_track(); // カーソルアウト対策用にホバー設定
            s.btn_idx_hover = sel_idx;
            if btn == MouseBtnState::NONE { s.btn_idx_push = -1; } // 初回選択からカーソルが外れたので左・右クリック選択対象は解除
            unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(old_idx - s.scroll_idx)), TRUE); }
            unsafe { InvalidateRect(s.handle, Some(&s.calc_idx2rect(s.btn_idx_hover - s.scroll_idx)), TRUE); }
        }

        let old_scroll = s.scroll_sel;
        s.scroll_sel =
            if sel_idx == -3 && s.scroll_idx > 0  { -3 }
            else if sel_idx == -4 && s.scroll_idx + s.scroll_num < s.num_item { -4 }
            else { -1 };

        if old_scroll != s.scroll_sel {
            s.hover_track();
            unsafe { InvalidateRect(s.handle, None, TRUE); }
        }
    }

    fn item_draw(&mut self, hdc: HDC) {
        let s = self;

        let (old_font, old_brs_clr, old_brs, old_pen) = unsafe {
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLOR_TEXT);
            (SelectObject(hdc, s.hfont.0),
            GetDCBrushColor(hdc),
            SelectObject(hdc, GetStockObject(DC_BRUSH)),
            SelectObject(hdc, GetStockObject(DC_PEN)))
        };

        let mut idx = 0;
        let mut grpidx = 0;
        let mut y = 0;
        for v in &s.vec_items {
            let mut count = 0;
            for i in v {
                if idx < s.scroll_idx {
                    idx += 1;
                    count += 1;
                    continue;
                }

                if v.len() > 1 || i.group_type == 2 {
                    let rc =
                    if i.group_type == 2 && v.len() == 1 {
                        RECT {
                            left: s.pad * 2,
                            top: y + s.item_height / 2 - s.group_bar_width,
                            right: s.pad * 2 + s.group_bar_width,
                            bottom: y + s.item_height / 2 + s.group_bar_width }

                    } else if count == 0 {
                        RECT {
                            left: s.pad * 2,
                            top: y + s.item_height / 2 - s.group_bar_width,
                            right: s.pad * 2 + s.group_bar_width,
                            bottom: y + s.item_height }

                    } else if count == v.len() - 1 {
                        RECT {
                            left: s.pad * 2,
                            top: y,
                            right: s.pad * 2+ s.group_bar_width,
                            bottom: y + s.item_height / 2 + s.group_bar_width }

                    } else {
                        RECT {
                            left: s.pad * 2,
                            top: y,
                            right: s.pad * 2 + s.group_bar_width,
                            bottom: y + s.item_height }
                    };

                    unsafe {
                        if i.group_type != 2 {
                            SetDCBrushColor(hdc, COLOR_GROUPBOX);
                            SetDCPenColor(hdc, COLOR_GROUPBOX);
                        } else {
                            SetDCBrushColor(hdc, COLOR_GROUPBOX_FREE);
                            SetDCPenColor(hdc, COLOR_GROUPBOX_FREE);
                        }
                        Rectangle(hdc, rc.left, rc.top, rc.right, rc.bottom);
                    }
                }

                if grpidx == s.grp_idx_push { unsafe {
                    SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                    SetDCPenColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                    let r = s.calc_idx2rect(idx - s.scroll_idx);
                    Rectangle(hdc, r.left, r.top, r.right, r.bottom);

                }} else if grpidx == s.grp_idx_sort_target { unsafe {
                    SetDCBrushColor(hdc, old_brs_clr);
                    SetDCPenColor(hdc, COLOR_TEXT);
                    if s.grp_idx_sort_target < s.grp_idx_push && count == 0 {
                        Rectangle(hdc, 0, y, s.wnd_width, y + 2);
                    } else if s.grp_idx_sort_target > s.grp_idx_push && count == v.len() - 1 {
                        Rectangle(hdc, 0, y + s.item_height - 2, s.wnd_width, y + s.item_height);
                    }

                }} else if idx == s.wnd_idx_push { unsafe {
                    SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                    SetDCPenColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                    let r = s.calc_idx2rect(idx - s.scroll_idx);
                    Rectangle(hdc, r.left, r.top, r.right, r.bottom);

                }} else if idx == s.wnd_idx_target { unsafe {
                    SetDCBrushColor(hdc, old_brs_clr);
                    SetDCPenColor(hdc, COLOR_TEXT);
                    if s.wnd_b_target_upper {
                        Rectangle(hdc, 0, y, s.wnd_width, y + 2);
                    } else {
                        Rectangle(hdc, 0, y + s.item_height - 2, s.wnd_width, y + s.item_height);
                    }

                }} else if idx == s.btn_idx_hover  { unsafe {
                    SetDCBrushColor(hdc, COLOR_CURSOR_HIGHLIGHT);
                    SetDCPenColor(hdc, COLOR_HIGHLIGHT_BORDER);
                    let r = s.calc_idx2rect(idx - s.scroll_idx);
                    Rectangle(hdc, r.left, r.top, r.right, r.bottom);
                }}

                unsafe { SetDCBrushColor(hdc, old_brs_clr); }
                let ii = s.map_icons.get(&i.handle.0);
                if ii.is_some() {
                    let _ = unsafe { DrawIconEx(hdc, s.pad * 2 + s.group_bar_width + s.pad * 2, y + s.pad, ii.unwrap().0, s.icon_width, s.icon_height, 0, None, DI_NORMAL) };
                }
                let mut rc = RECT{ left: s.pad * 2 + s.group_bar_width + s.pad * 2 + s.icon_width + s.pad, top: y, right: s.wnd_width - s.pad, bottom: y + s.item_height};
                unsafe { DrawTextExW(hdc, &mut WSTR::from(&i.title).0,  &mut rc, DT_TOP | DT_VCENTER |  DT_SINGLELINE |  DT_NOPREFIX | DT_PATH_ELLIPSIS | DT_END_ELLIPSIS, None); }
                y += s.item_height;
                idx += 1;
                count += 1;
                if idx >= s.scroll_idx + s.scroll_num { break; }
            }
            if idx >= s.scroll_idx + s.scroll_num { break; }
            grpidx += 1;
        }

        unsafe {
            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brs);
            SelectObject(hdc, old_font);
        }
    }

    fn scrollbar_draw(&mut self, hdc: HDC) {
        let s = self;

        let y_base = s.scroll_num * s.item_height;
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
                    if s.scroll_idx + s.scroll_num - 1 < s.num_item - 1 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
                SetDCBrushColor(hdc,
                    if s.scroll_idx + s.scroll_num - 1 < s.num_item - 1 { COLOR_TEXT } else { COLOR_SCROLLBAR_BORDER });
            }
            Polygon(hdc, &pt);

            SelectObject(hdc, old_pen);
            SelectObject(hdc, old_brs);
        }
    }
}

impl WndMsgHandler for WindowViewWnd {
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

                    if s.btn_idx_hover <= -3 {
                        s.scroll_chk(lparam.0 as i32 & u16::MAX as i32, lparam.0 as i32 >> u16::BITS, false);

                    } else if s.grp_idx_push >= 0 {
                        s.sort_group();

                    } else if s.btn_idx_push == s.btn_idx_hover && s.btn_idx_hover >= 0 {
                        let h = s.app().main_wnd().handle(); // destroyが走ってもいいように先にハンドルを取得しておく
                        let r = s.item_handle(POINT { x:lparam.0 as i32 & u16::MAX as i32, y:lparam.0 as i32 >> u16::BITS }, false);
                        if r.is_ok() {
                            let _ = unsafe { PostMessageW(h, WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
                        }
                    }
                    (s.btn_idx_push, s.btn_idx_hover) = (-1, -1);
                    (s.grp_idx_push, s.grp_idx_sort_target) = (-1, -1);
                    unsafe { InvalidateRect(s.handle, None, TRUE); }
                }
            }
            WM_MBUTTONUP => {
                if let MouseBtnState::MDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    if s.wnd_idx_push >= 0 {
                        s.sort_window();
                    }
                    (s.wnd_idx_push, s.wnd_idx_target) = (-1, -1);
                    unsafe { InvalidateRect(s.handle, None, TRUE) };
                }
            }
            WM_RBUTTONUP => {
                if let MouseBtnState::RDOWN = s.btn_state {
                    let _ = unsafe { ReleaseCapture() };
                    s.btn_state = MouseBtnState::NONE;

                    if s.btn_idx_push == s.btn_idx_hover && s.btn_idx_hover != -1 {
                        let h = s.app().main_wnd().handle(); // destroyが走ってもいいように先にハンドルを取得しておく
                        let r = s.item_handle(POINT { x:lparam.0 as i32 & u16::MAX as i32, y:lparam.0 as i32 >> u16::BITS }, true);
                        if r.is_ok() {
                            let _ = unsafe { PostMessageW(h, WMU_WINCLOSE, WPARAM(0), LPARAM(0)) };
                        }
                    }
                    (s.btn_idx_push, s.btn_idx_hover) = (-1, -1);
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
            }
            WM_MOUSEWHEEL => {
                if s.scroll_num < s.num_item {
                    let delta = (wparam.0 >> u16::BITS) as i16 / WHEEL_DELTA as i16;
                    s.scroll_do(delta as i32);
                }
                return Some(LRESULT(0))
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = unsafe { BeginPaint(s.handle, &mut ps) };

                s.item_draw(hdc);

                if s.scroll_num < s.num_item {
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
            }
            WM_DESTROY => {
                s.app().main_wnd().get_mut().vec_window_items = s.vec_items.clone();
            }
            _ => { }
        }
        None
    }
}
