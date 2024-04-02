use std::{rc::{Rc, Weak}, sync::Once};
use fxhash::FxHashMap;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{Controls::{SetScrollInfo, BST_CHECKED, BST_UNCHECKED}, HiDpi::{AdjustWindowRectExForDpi, GetDpiForMonitor, GetDpiForWindow, GetSystemMetricsForDpi, SystemParametersInfoForDpi, MDT_EFFECTIVE_DPI}, Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled}, Shell::{DefSubclassProc, SetWindowSubclass}, WindowsAndMessaging::* },
    }
};

use crate::{lib_common::{wnd_instance, wnd_proc, RcValueRef, WndMsgHandler, WSTR}, Font};

static ONCE: Once = Once::new();

const CHAR_FONT_HEIGHT_MEASURE: &str = "|";
pub const CHAR_FONT_WIDTH_MEASURE: &str = "W"; // アルファベット一文字当たりの幅(最も幅をとる文字)

#[derive(Default)]
pub struct WindowContainer {
    handle: HWND,
    cont_w: i32,
    cont_h: i32,
    m_child: FxHashMap<isize /*HWND*/, ChildHolder>,
    l_child: Vec<HWND>,
    l_child_container: Vec<HWND>,
    b_recalc_layout_stop: bool,
    scr_v: SCROLLINFO,
    scr_h: SCROLLINFO,
    layout: Layout,
    msg_proc: Option<Box<dyn WindowContainerMsgProc>>,
    sub_proc: Option<Box<dyn WindowContainerSubProc>>,
}

pub type WindowContainerWeak = Weak<WindowContainer>;
pub type WindowContainerRc = Rc<WindowContainer>;

struct ChildHolderPlace {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    pos_offset_hwnd: HWND,
    span_offset_hwnd: HWND,
    pos_kind: PlaceSet,
    span_kind: PlaceSet,
}

struct ChildHolderVStack {
    w: i32,
    h: i32,
    pad: i32,
    filler: i32,
    split: i32,
    b_auto: bool,
    align: AlignH,
    vert: HeightAuto,
}

struct ChildHolderHStack {
    w: i32,
    h: i32,
    pad: i32,
    filler: i32,
    split: i32,
    b_auto: bool,
    align: AlignV,
    horz: WidthAuto,
}

enum ChildHolder {
    Place(ChildHolderPlace), VStack(ChildHolderVStack), HStack(ChildHolderHStack)
}

#[derive(Default)]
pub enum Layout { #[default] None, Place, VStack, HStack }

#[allow(unused)]
#[derive(Default, Clone)]
pub enum AlignV { #[default] TOP, CENTER, BOTTOM, FILL, EXPAND }

#[allow(unused)]
#[derive(Default, Clone)]
pub enum AlignH { #[default] LEFT, CENTER, RIGHT, FILL, EXPAND }

#[allow(unused)]
#[derive(Default, Clone)]
pub enum HeightAuto { #[default] AUTO, FIX }

#[allow(unused)]
#[derive(Default, Clone)]
pub enum WidthAuto { #[default] AUTO, FIX }

#[allow(unused)]
#[derive(Default, Clone)]
pub enum PlaceSet { #[default] PIXEL, REL, OFFSET }

pub trait WindowContainerMsgProc {
    #[allow(unused_variables)]
    fn msgproc(&mut self, hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> { None }
}
pub trait WindowContainerSubProc {
    #[allow(unused_variables)]
    fn subclassproc(&mut self, hwnd: HWND, child_id: usize, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> { None }
}

impl RcValueRef<WindowContainer> for WindowContainerRc {}

pub trait WContainerBehavior {
    fn create(style_ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, h_brush: HBRUSH, h_cursor: HCURSOR, x: i32, y: i32, w: i32, h: i32, hparent: HWND, cmdid: HMENU) -> WindowContainerWeak;
    fn create_child_container(&mut self, style_ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, cmdid: HMENU) -> WindowContainerWeak;
    fn set_msg_proc(&mut self, m: Option<Box<dyn WindowContainerMsgProc>>);
    fn set_sub_proc(&mut self, m: Option<Box<dyn WindowContainerSubProc>>);

    fn create_child(&mut self, style_ex: WINDOW_EX_STYLE, clsname: PCWSTR, wndname: PCWSTR, style: WINDOW_STYLE, cmdid: HMENU, b_subclass: bool) -> HWND;
    fn remove_child(&mut self, hwnd: HWND);
    fn get_child_rect(&self, htarget: HWND) -> (i32, i32, i32, i32) /* Left, Top, Right, Bottom */;
    fn get_field_size(&self) -> (i32, i32) /* width, height */;
    fn get_scrollpos(&self) -> (i32, i32) /* nPosH, nPosV */;

    fn recalc_layout(&mut self);
    fn recalc_layout_stop(&mut self, b: bool);
    fn check_layout(&mut self) -> (i32, i32);
    fn update_layout(&mut self);

    fn vstack(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, align: AlignH, size: HeightAuto);
    fn get_vstack_param(&self, htarget: HWND) -> (i32, i32, i32, i32, i32, bool, AlignH, HeightAuto);
    fn set_vstack_param(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, align: AlignH, size: HeightAuto);
    fn layout_vstack(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32);

    fn hstack(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, size: WidthAuto, align: AlignV);
    fn get_hstack_param(&self, htarget: HWND) -> (i32, i32, i32, i32, i32, bool, WidthAuto, AlignV);
    fn set_hstack_param(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, size: WidthAuto, align: AlignV);
    fn layout_hstack(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32);

    fn place(&mut self, htarget: HWND, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind: PlaceSet, span_kind: PlaceSet);
    fn get_place_param(&self, htarget: HWND) -> (f64, f64, f64, f64, HWND, HWND, PlaceSet, PlaceSet);
    fn set_place_param(&mut self, htarget: HWND, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind: PlaceSet, span_kind: PlaceSet);
    fn layout_place(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32);

    // fn grid
    // fn get_grid_param
    // fn set_grid_param
    // fn layout_grid(&mut self, b_min_check: bool, nowScrV: i32, nowScrH: i32) -> (i32, i32);

}

impl WContainerBehavior for WindowContainerRc {
    /** ウィンドウクラスは使いまわしとなるため、2回目以降のブラシ、カーソル指定は無効 */
    fn create(style_ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, h_brush: HBRUSH, h_cursor: HCURSOR, x: i32, y: i32, w: i32, h: i32, hparent: HWND, cmdid: HMENU) -> WindowContainerWeak {
        let mut rc = Rc::new(WindowContainer {
            cont_w: w,
            cont_h: h,
            ..Default::default()
        });
        rc.get_mut().scr_v.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
        rc.get_mut().scr_h.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;

        let window_class = w!("window_container");
        ONCE.call_once(|| {
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpszClassName: window_class,
                hbrBackground: h_brush,
                hCursor: h_cursor,
                lpfnWndProc: Some(wnd_proc::<WindowContainer>),
                ..Default::default()
            };
            unsafe { RegisterClassExW(&wc) };
        });

        // wnd.handleはWM_NCCREATEの処理の中で設定される
        unsafe { CreateWindowExW(style_ex, window_class, w!(""), style | WS_CHILD | WS_CLIPCHILDREN,
            x, y, w, h, hparent, cmdid, None, Some(&rc as *const _ as _)) };

        Rc::downgrade(&rc)
    }

    fn create_child_container(&mut self, style_ex: WINDOW_EX_STYLE, style: WINDOW_STYLE, cmdid: HMENU) -> WindowContainerWeak {
        let wcw = Self::create(style_ex, style, HBRUSH(0), HCURSOR(0), 0, 0, 0, 0, self.handle, cmdid);
        self.get_mut().l_child_container.push(wcw.upgrade().unwrap().handle());
        wcw
    }

    fn set_msg_proc(&mut self, m: Option<Box<dyn WindowContainerMsgProc>>) {
        self.get_mut().msg_proc = m;
    }

    fn set_sub_proc(&mut self, m: Option<Box<dyn WindowContainerSubProc>>) {
        self.get_mut().sub_proc = m;
    }

    fn create_child(&mut self, style_ex: WINDOW_EX_STYLE, clsname: PCWSTR, wndname: PCWSTR, style: WINDOW_STYLE, cmdid: HMENU, b_subclass: bool) -> HWND {
        let hwnd = unsafe { CreateWindowExW(style_ex, clsname, wndname, style | WS_CHILD,
            0, 0, 0, 0, self.handle, cmdid, None, None) };
        if b_subclass && self.sub_proc.is_some() {
            unsafe { SetWindowSubclass(hwnd, Some(child_subclass_proc), cmdid.0 as usize, Rc::as_ptr(self) as _); }
        }
        hwnd
    }

    fn remove_child(&mut self, hwnd: HWND) {
        let s = self.get_mut();
        for (i, h) in s.l_child.iter().enumerate() {
            if h.eq(&hwnd) {
                s.l_child.remove(i);
                break;
            }
        }
        for (i, h) in s.l_child_container.iter().enumerate() {
            if h.eq(&hwnd) {
                s.l_child_container.remove(i);
                break;
            }
        }
        s.m_child.remove(&hwnd.0).and_then(|_| Some(unsafe { DestroyWindow(hwnd) }));
    }

    // コンテナ左上を0,0にした時の子ウィンドウの位置オフセットを返す(コンテナの大きさを超えることあり。スクロール状態は戻り値に無関係)
    fn get_child_rect(&self, htarget: HWND) -> (i32, i32, i32, i32) /* Left, Top, Right, Bottom */ {
        let mut rc = RECT::default();
		let _ = unsafe { GetWindowRect(self.handle, &mut rc) };
        let mut rct = RECT::default();
		let _ = unsafe { GetWindowRect(htarget, &mut rct) };
        (rct.left - rc.left + self.scr_h.nPos, rct.top - rc.top + self.scr_v.nPos,
            rct.left - rc.left + self.scr_h.nPos + rct.right - rct.left, rct.top - rc.top + self.scr_v.nPos + rct.bottom - rct.top)
    }

    fn get_field_size(&self) -> (i32, i32) /* width, height */ {
        (self.scr_h.nMax + 1, self.scr_v.nMax + 1)
    }

    fn get_scrollpos(&self) -> (i32, i32) /* nPosH, nPosV */ {
        (self.scr_h.nPos, self.scr_v.nPos)
    }

    /** 子コンテナの含むアイテムのminサイズを再帰的に再計算し、各コンテナの大きさを変更する */
    fn recalc_layout(&mut self) {
        if self.b_recalc_layout_stop { return }
        (self.get_mut().scr_h.nPos, self.get_mut().scr_v.nPos) = (0, 0);
        let v = self.l_child_container.clone(); // 二重借用回避(self.set_xstack_paramと)
        for h in v {
            let w = wnd_instance::<WindowContainer>(h);
            if let Some(mut c) = w.upgrade() {
                c.recalc_layout();
                if let Layout::VStack = self.layout {
                    let (width, height) = c.check_layout();
                    let p = self.get_vstack_param(h);
                    self.set_vstack_param(h, width, height, p.2, p.3, p.4, p.5, p.6, p.7);
                } else if let Layout::HStack = self.layout {
                    let (width, height) = c.check_layout();
                    let p = self.get_hstack_param(h);
                    self.set_hstack_param(h, width, height, p.2, p.3, p.4, p.5, p.6, p.7);
                } else if let Layout::Place = self.layout {
                    let (width, height) = c.check_layout();
                    let p = self.get_place_param(h);
                    self.set_place_param(h, p.0, p.1, width as f64, height as f64, p.4, HWND(0), p.6, PlaceSet::PIXEL);
                }
            }
        }
    }

    fn recalc_layout_stop(&mut self, b: bool) {
        self.get_mut().b_recalc_layout_stop = b;
    }

    fn check_layout(&mut self) -> (i32, i32) {
        let mut sc = self.clone();
        let s = self.get_mut();

        match s.layout {
            Layout::None => { (0, 0) }
            Layout::VStack => { sc.layout_vstack(true, 0, 0) }
            Layout::HStack => { sc.layout_hstack(true, 0, 0) }
            Layout::Place => { sc.layout_place(true, 0, 0) }
        }
    }

    fn update_layout(&mut self) {
        let mut sc = self.clone();
        let s = self.get_mut();

        unsafe { DefWindowProcW(s.handle,WM_SETREDRAW, WPARAM(FALSE.0 as usize), None); }

        let (now_v, now_h) = (s.scr_v.nPos, s.scr_h.nPos);

        let (view_w, view_h) = match s.layout {
            Layout::None => { (0, 0) }
            Layout::VStack => { sc.layout_vstack(false, now_v, now_h) }
            Layout::HStack => { sc.layout_hstack(false, now_v, now_h) }
            Layout::Place => { sc.layout_place(false, now_v, now_h) }
        };

        // スクロール処理
        let (mut b_add_vscr, mut b_add_hscr) = (false, false);
        let (mut cont_w, mut cont_h) = (s.cont_w, s.cont_h);
        for _ in 0..2 { // 初回の計算で一方にスクロールバーが生じ、もともと収まっていた他方が影響を受け両方を表示する必要があるケースへの対応
            s.scr_h.fMask = SIF_ALL;
            s.scr_h.nMax = view_w - 1;
            s.scr_h.nPage = cont_w as u32;
            if view_w > cont_w {
                cont_h -= if !b_add_hscr { unsafe { GetSystemMetricsForDpi(SM_CYHSCROLL, GetDpiForWindow(s.handle)) } } else { 0 };
                b_add_hscr = true;
            }
            if s.scr_h.nMax <= s.scr_h.nPage as i32 { s.scr_h.nPos = 0; }

            s.scr_v.fMask = SIF_ALL;
            s.scr_v.nMax = view_h - 1;
            s.scr_v.nPage = cont_h as u32;
            if view_h > cont_h {
                cont_w -= if !b_add_vscr { unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, GetDpiForWindow(s.handle)) } } else { 0 };
                b_add_vscr = true;
            }
            if s.scr_v.nMax <= s.scr_v.nPage as i32 { s.scr_v.nPos = 0; }
        }
        unsafe { SetScrollInfo(s.handle, SB_HORZ, &s.scr_h, TRUE); }
        unsafe { SetScrollInfo(s.handle, SB_VERT, &s.scr_v, TRUE); }

        s.scr_h.fMask = SIF_POS;
        s.scr_v.fMask = SIF_POS;
        let _ = unsafe { GetScrollInfo(s.handle, SB_HORZ, &mut s.scr_h) };
        let _ = unsafe { GetScrollInfo(s.handle, SB_VERT, &mut s.scr_v) };
        unsafe { ScrollWindowEx(s.handle, now_h - s.scr_h.nPos, now_v - s.scr_v.nPos, None, None, None, None, SW_SCROLLCHILDREN); }

        unsafe { DefWindowProcW(s.handle, WM_SETREDRAW, WPARAM(TRUE.0 as usize), None); }
        unsafe { RedrawWindow(s.handle, None, None, RDW_ERASE | RDW_INVALIDATE | RDW_ALLCHILDREN); }
    }

    fn vstack(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, align: AlignH, size: HeightAuto) {
        let s = self.get_mut();

        if let Layout::None = s.layout {
            s.layout = Layout::VStack;
        }

        if let Layout::VStack = s.layout {
            let ch = ChildHolderVStack {
                w: w,
                h: h,
                pad: pad,
                filler: filler,
                split: split,
                b_auto,
                align: align,
                vert: size,
            };
            s.m_child.insert(htarget.0, ChildHolder::VStack(ch));
            s.l_child.push(htarget);

            // self.update_layout();
        }
    }

    fn get_vstack_param(&self, htarget: HWND) -> (i32, i32, i32, i32, i32, bool, AlignH, HeightAuto) {
        let ch = self.m_child.get(&htarget.0);
        if let Some(ChildHolder::VStack (v)) = ch {
            (v.w, v.h, v.pad, v.filler, v.split, v.b_auto, v.align.clone(), v.vert.clone())
        } else {
            debug_assert!(false);
            (0, 0, 0, 0, 0, true, AlignH::default(), HeightAuto::default())
        }
    }

    fn set_vstack_param(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, align: AlignH, size: HeightAuto) {
        let s = self.get_mut();

        let ch = s.m_child.get(&htarget.0);
        if ch.is_none() {
            debug_assert!(false);
            return;
        }

        let ch = ChildHolder::VStack(ChildHolderVStack {
            w: w,
            h: h,
            pad: pad,
            filler: filler,
            split: split,
            b_auto,
            align: align,
            vert: size
        });
        s.m_child.insert(htarget.0, ch);
        self.update_layout();
    }

    fn layout_vstack(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32) {
        let s = self.get_mut();

        // 子コントロールのサイズをすべて計算し、ビューサイズを計算(一番右下はどこか)
        let mut view_w = 0i32;
        let mut view_h = 0i32;

        let mut size_h_min = 0i32;
        let mut num_auto_elem = 0i32;

        let mut b_expand = false;
        for hc in s.l_child.iter() { // サイズ固定分の積み上げ、自動サイズの要素の個数を確認
            let ch = s.m_child.get(&hc.0).unwrap();
            if let ChildHolder::VStack(v) = ch {
                size_h_min += v.h + v.pad * 2 + v.split; // 固定でも自動でもv.hには最低サイズが入っている
                // if let HeightAuto::FIX = v.vert  { }
                if let HeightAuto::AUTO = v.vert {
                    if v.b_auto {
                        num_auto_elem += 1;
                    } else {
                        size_h_min += v.filler;
                    }
                }
                if v.w + v.pad * 2 > view_w { view_w = v.w + v.pad * 2; }
                if let AlignH::EXPAND = v.align { b_expand = true; }
            }
        }

        if b_min_check {
            return (view_w, size_h_min)
        }

        // MINサイズの積み上げでビューの方が小さければコンテナのサイズまで広げる
        if b_expand && s.cont_w > view_w { view_w = s.cont_w; }

        // コンテナの空きサイズ、自動サイズ要素ひとつあたりのFillerサイズ(高さor幅)を計算
        let mut auto_filler = 0i32;
        let mut size_free = s.cont_h - size_h_min;
        if size_free > 0 {
            if num_auto_elem != 0 {
                auto_filler = size_free / num_auto_elem;
            }
        } else {
            size_free = 0;
        }

        let mut now = 0i32;
        let mut now_auto_count = 0i32;

        for hc in s.l_child.iter() { // 各要素の配置計算、SetWinPos、ビューサイズの積み上げ
            let ch = s.m_child.get(&hc.0).unwrap();
            if let ChildHolder::VStack(v) = ch {
                let (newx, neww) =
                    match v.align {
                        AlignH::LEFT => (v.pad, v.w),
                        AlignH::RIGHT => (view_w - v.pad - v.w, v.w),
                        AlignH::CENTER => (if view_w / 2 - v.w / 2 > 0 { view_w / 2 - v.w / 2 } else { v.pad }, v.w),
                        AlignH::FILL | AlignH::EXPAND => (v.pad, view_w - v.pad * 2),
                    };
                let newy = now + v.pad;
                let newh = match v.vert {
                    HeightAuto::FIX => { v.h }
                    HeightAuto::AUTO => {
                        if !v.b_auto {
                            v.h + v.filler
                        } else {
                            now_auto_count += 1;
                            if now_auto_count == num_auto_elem { // 自動サイズ最後の要素は、auto_size整数除算の剰余をうめないと隙間が出る
                                v.h + size_free
                            } else {
                                size_free -= auto_filler;
                                v.h + auto_filler
                            }
                        }
                    }
                };
                let _ = unsafe { SetWindowPos(*hc, None, newx - now_scr_h, newy - now_scr_v, neww, newh, SWP_NOZORDER) };
                now += newh + v.pad * 2 + v.split;
                if view_h < now { view_h = now; }
            }
        }
        (view_w, view_h)
    }

    fn hstack(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, size: WidthAuto, align: AlignV) {
        let s = self.get_mut();

        if let Layout::None = s.layout {
            s.layout = Layout::HStack;
        }

        if let Layout::HStack = s.layout {
            let ch = ChildHolderHStack {
                w: w,
                h: h,
                pad: pad,
                filler: filler,
                split: split,
                b_auto,
                align: align,
                horz: size,
            };
            s.m_child.insert(htarget.0, ChildHolder::HStack(ch));
            s.l_child.push(htarget);

            // self.update_layout();
        }
    }

    fn get_hstack_param(&self, htarget: HWND) -> (i32, i32, i32, i32, i32, bool, WidthAuto, AlignV) {
        let ch = self.m_child.get(&htarget.0);
        if let Some(ChildHolder::HStack (v)) = ch {
            (v.w, v.h, v.pad, v.filler, v.split, v.b_auto, v.horz.clone(), v.align.clone())
        } else {
            debug_assert!(false);
            (0, 0, 0, 0, 0, true, WidthAuto::default(), AlignV::default())
        }
    }

    fn set_hstack_param(&mut self, htarget: HWND, w: i32, h: i32, pad: i32, filler: i32, split: i32, b_auto: bool, size: WidthAuto, align: AlignV) {
        let s = self.get_mut();

        let ch = s.m_child.get(&htarget.0);
        if ch.is_none() {
            debug_assert!(false);
            return;
        }

        let ch = ChildHolder::HStack(ChildHolderHStack {
            w: w,
            h: h,
            pad: pad,
            filler: filler,
            split: split,
            b_auto,
            align: align,
            horz: size
        });
        s.m_child.insert(htarget.0, ch);
        self.update_layout();
    }

    fn layout_hstack(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32) {
        let s = self.get_mut();

        // 子コントロールのサイズをすべて計算し、ビューサイズを計算(一番右下はどこか)
        let mut view_w = 0i32;
        let mut view_h = 0i32;

        let mut size_w_min = 0i32;
        let mut num_auto_elem = 0i32;

        let mut b_expand = false;
        for hc in s.l_child.iter() { // サイズ固定分の積み上げ、自動サイズの要素の個数を確認
            let ch = s.m_child.get(&hc.0).unwrap();
            if let ChildHolder::HStack(v) = ch {
                size_w_min += v.w + v.pad * 2 + v.split; // 固定でも自動でもv.wには最低サイズが入っている
                // if let WidthAuto::FIX = v.vert  { }
                if let WidthAuto::AUTO = v.horz {
                    if v.b_auto {
                        num_auto_elem += 1;
                    } else {
                        size_w_min += v.filler;
                    }
                }
                if v.h + v.pad * 2 > view_h { view_h = v.h + v.pad * 2; }
                if let AlignV::EXPAND = v.align { b_expand = true; }
            }
        }

        if b_min_check {
            return (size_w_min, view_h)
        }

        // MINサイズの積み上げでビューの方が小さければコンテナのサイズまで広げる
        if b_expand && s.cont_h > view_h { view_h = s.cont_h; }

        // コンテナの空きサイズ、自動サイズ要素ひとつあたりのFillerサイズ(高さor幅)を計算
        let mut auto_filler = 0i32;
        let mut size_free = s.cont_w - size_w_min;
        if size_free > 0 {
            if num_auto_elem != 0 {
                auto_filler = size_free / num_auto_elem;
            }
        } else {
            size_free = 0;
        }

        let mut now = 0i32;
        let mut now_auto_count = 0i32;

        for hc in s.l_child.iter() { // 各要素の配置計算、SetWinPos、ビューサイズの積み上げ
            let ch = s.m_child.get(&hc.0).unwrap();
            if let ChildHolder::HStack(v) = ch {
                let (newy, newh) =
                    match v.align {
                        AlignV::TOP => (v.pad, v.h),
                        AlignV::BOTTOM => (view_h - v.pad - v.h, v.h),
                        AlignV::CENTER => (if view_h / 2 - v.h / 2 > 0 { view_h / 2 - v.h / 2 } else { v.pad }, v.h),
                        AlignV::FILL | AlignV::EXPAND => (v.pad, view_h - v.pad * 2),
                    };
                let newx = now + v.pad;
                let neww = match v.horz {
                    WidthAuto::FIX => { v.w }
                    WidthAuto::AUTO => {
                        if !v.b_auto {
                            v.w + v.filler
                        } else {
                            now_auto_count += 1;
                            if now_auto_count == num_auto_elem { // 自動サイズ最後の要素は、auto_size整数除算の剰余をうめないと隙間が出る
                                v.w + size_free
                            } else {
                                size_free -= auto_filler;
                                v.w + auto_filler
                            }
                        }
                    }
                };
                let _ = unsafe { SetWindowPos(*hc, None, newx - now_scr_h, newy - now_scr_v, neww, newh, SWP_NOZORDER) };
                now += neww + v.pad * 2 + v.split;
                if view_w < now { view_w = now; }
            }
        }
        (view_w, view_h)
    }

    fn place(&mut self, htarget: HWND, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind: PlaceSet, span_kind: PlaceSet) {
        let s = self.get_mut();

        if let Layout::None = s.layout {
            s.layout = Layout::Place;
        }

        if let Layout::Place = s.layout {
            let ch = ChildHolderPlace {
                x: x,
                y: y,
                w: w,
                h: h,
                pos_offset_hwnd: pos_offset_hwnd,
                span_offset_hwnd: span_offset_hwnd,
                pos_kind:pos_kind,
                span_kind: span_kind,
            };
            s.m_child.insert(htarget.0, ChildHolder::Place(ch));
            s.l_child.push(htarget);

            // self.update_layout();
        }
    }
    fn get_place_param(&self, htarget: HWND) -> (f64, f64, f64, f64, HWND, HWND, PlaceSet, PlaceSet) {
        let ch = self.m_child.get(&htarget.0);
        if let Some(ChildHolder::Place (v)) = ch {
            (v.x, v.y, v.w, v.h, v.pos_offset_hwnd, v.span_offset_hwnd, v.pos_kind.clone(), v.span_kind.clone())
        } else {
            debug_assert!(false);
            (0.0, 0.0, 0.0, 0.0, HWND(0), HWND(0), PlaceSet::PIXEL, PlaceSet::PIXEL)
        }
    }
    fn set_place_param(&mut self, htarget: HWND, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind: PlaceSet, span_kind: PlaceSet) {
        let s = self.get_mut();

        let ch = s.m_child.get(&htarget.0);
        if ch.is_none() {
            debug_assert!(false);
            return;
        }

        let ch = ChildHolder::Place(ChildHolderPlace {
            x: x,
            y: y,
            w: w,
            h: h,
            pos_offset_hwnd: pos_offset_hwnd,
            span_offset_hwnd: span_offset_hwnd,
            pos_kind:pos_kind,
            span_kind: span_kind,
        });
        s.m_child.insert(htarget.0, ch);
        self.update_layout();
    }
    fn layout_place(&mut self, b_min_check: bool, now_scr_v: i32, now_scr_h: i32) -> (i32, i32) {
        // let s = self.get_mut();

        let place_calc_pos = |v :&ChildHolderPlace, pw :i32, ph :i32| -> (i32, i32, i32, i32) {
            let (newx, newy) = match v.pos_kind {
                PlaceSet::PIXEL => (v.x.round() as i32, v.y.round() as i32),
                PlaceSet::REL => ((pw as f64 * v.x).round() as i32, (ph as f64 * v.y).round() as i32),
                PlaceSet::OFFSET => {
                    let (x, y, _, _) = self.get_child_rect(v.pos_offset_hwnd);
                    (x + v.x.round() as i32, y + v.y.round() as i32)
                }
            };
            let (neww, newh) = match v.span_kind {
                PlaceSet::PIXEL => (v.w.round() as i32, v.h.round() as i32),
                PlaceSet::REL => ((pw as f64 * v.w).round() as i32, (ph as f64 * v.h).round() as i32),
                PlaceSet::OFFSET => {
                    let (_, _, w, h) = self.get_child_rect(v.span_offset_hwnd);
                    (w + v.w.round() as i32, h + v.h.round() as i32)
                }
            };
            (newx, newy, neww, newh)
        };

        // 子コントロールのサイズをすべて計算し、ビューサイズを計算(一番右下はどこか)
        let mut view_w = 0i32;
        let mut view_h = 0i32;

        for hc in self.l_child.iter() {
            let ch = self.m_child.get(&hc.0).unwrap();
            if let ChildHolder::Place(v) = ch {
                let (newx, newy, neww, newh) = place_calc_pos(&v, self.cont_w, self.cont_h);
                if !b_min_check {
                    let _ = unsafe { SetWindowPos(*hc, None, newx - now_scr_h, newy - now_scr_v, neww, newh, SWP_NOZORDER) };
                }
                if view_w < newx + neww { view_w = newx + neww; }
                if view_h < newy + newh { view_h = newy + newh; }
            }
        }
        (view_w, view_h)
    }
}

extern "system" fn child_subclass_proc(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM, uidsubclass: usize, dwrefdata: usize) -> LRESULT {
    let inst = unsafe { &mut *(dwrefdata as *mut WindowContainer) };
    if inst.sub_proc.is_some() {
        if let Some(r) = inst.sub_proc.as_mut().unwrap().subclassproc(hwnd, uidsubclass, umsg, wparam, lparam) { return r }
    }
    unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) }
}

impl WndMsgHandler for WindowContainer {
    fn handle(&self) -> HWND {
        self.handle
    }

    fn set_handle(&mut self, h: HWND) {
        self.handle = h;
    }

    fn message_handler(&mut self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        let s = self;

        if s.msg_proc.is_some() {
            if let Some(r) = s.msg_proc.as_mut().unwrap().msgproc(s.handle, umsg, wparam, lparam) { return Some(r) }
        }

        match umsg {
            WM_SIZE => {
                let mut rc = RECT::default();
                let _ = unsafe { GetWindowRect(s.handle, &mut rc) };
                (s.cont_w, s.cont_h) = (rc.right - rc.left, rc.bottom - rc.top); // スクロールバーを含むコンテナウィンドウのサイズ
                wnd_instance::<Self>(s.handle).upgrade().unwrap().update_layout();
            }
            WM_HSCROLL => {
                let now_h = s.scr_h.nPos;
                let now_v = s.scr_v.nPos;
                match SCROLLBAR_COMMAND((wparam.0 & u16::MAX as usize) as i32) {
                    SB_TOP => { s.scr_h.nPos = s.scr_h.nMin; }
                    SB_BOTTOM => { s.scr_h.nPos = s.scr_h.nMax - s.scr_h.nPage as i32 + 1; }
                    SB_LINEUP => { if s.scr_h.nMin < s.scr_h.nPos { s.scr_h.nPos -= 1; }}
                    SB_LINEDOWN => { if s.scr_h.nPos < s.scr_h.nMax - s.scr_h.nPage as i32 + 1 { s.scr_h.nPos += 1; }}
                    SB_PAGEUP => {
                        s.scr_h.nPos -= s.scr_h.nPage as i32;
                        if s.scr_h.nPos < s.scr_h.nMin { s.scr_h.nPos = s.scr_h.nMin; }}
                    SB_PAGEDOWN => {
                        s.scr_h.nPos += s.scr_h.nPage as i32;
                        if s.scr_h.nMax - s.scr_h.nPage as i32 + 1 < s.scr_h.nPos { s.scr_h.nPos = s.scr_h.nMax - s.scr_h.nPage as i32 + 1; }}
                    SB_THUMBTRACK => { s.scr_h.nPos = (wparam.0 as u32 >> u16::BITS) as i32; }
                    SB_THUMBPOSITION => { s.scr_h.nPos = (wparam.0 as u32 >> u16::BITS) as i32; }
                    _ => {}
                }
                unsafe {
                    SetScrollInfo(s.handle , SB_HORZ , &s.scr_h , TRUE);
                    let mut rc = RECT::default();
                    ScrollWindowEx(s.handle, now_h - s.scr_h.nPos, now_v - s.scr_v.nPos, None, None, None, Some(&mut rc), SW_SCROLLCHILDREN);
                    RedrawWindow(s.handle, Some(&rc), None, RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN);
                }
            }
            WM_VSCROLL => {
                let now_h = s.scr_h.nPos;
                let now_v = s.scr_v.nPos;
                match SCROLLBAR_COMMAND((wparam.0 & u16::MAX as usize) as i32) {
                    SB_TOP => { s.scr_v.nPos = s.scr_v.nMin; }
                    SB_BOTTOM => { s.scr_v.nPos = s.scr_v.nMax - s.scr_v.nPage as i32 + 1; }
                    SB_LINEUP => { if s.scr_v.nMin < s.scr_v.nPos { s.scr_v.nPos -= 1; }}
                    SB_LINEDOWN => { if s.scr_v.nPos < s.scr_v.nMax - s.scr_v.nPage as i32 + 1 { s.scr_v.nPos += 1; }}
                    SB_PAGEUP => {
                        s.scr_v.nPos -= s.scr_v.nPage as i32;
                        if s.scr_v.nPos < s.scr_v.nMin { s.scr_v.nPos = s.scr_v.nMin; }}
                    SB_PAGEDOWN => {
                        s.scr_v.nPos += s.scr_v.nPage as i32;
                        if s.scr_v.nMax - s.scr_v.nPage as i32 + 1 < s.scr_v.nPos { s.scr_v.nPos = s.scr_v.nMax - s.scr_v.nPage as i32 + 1; }}
                    SB_THUMBTRACK => { s.scr_v.nPos = (wparam.0 as u32 >> u16::BITS) as i32; }
                    SB_THUMBPOSITION => { s.scr_v.nPos = (wparam.0 as u32 >> u16::BITS) as i32; }
                    _ => {}
                }
                unsafe {
                    SetScrollInfo(s.handle, SB_VERT , &s.scr_v , TRUE);
                    let mut rc = RECT::default();
                    ScrollWindowEx(s.handle, now_h - s.scr_h.nPos, now_v - s.scr_v.nPos, None, None, None, Some(&mut rc), SW_SCROLLCHILDREN);
                    RedrawWindow(s.handle, Some(&rc), None, RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN);
                }
            }
            WM_MOUSEHWHEEL => {
                if s.scr_h.nMax + 1 > s.scr_h.nPage as i32 {
                    let delta = (wparam.0 >> u16::BITS) as i16 / WHEEL_DELTA as i16;
                    let mut npos = s.scr_h.nPos + (delta * (s.scr_h.nPage / 10) as i16) as i32;
                    if npos < s.scr_h.nMin { npos = s.scr_h.nMin; }
                    if npos > s.scr_h.nMax - s.scr_h.nPage as i32 + 1 { npos = s.scr_h.nMax - s.scr_h.nPage as i32 + 1; }
                    unsafe { SendMessageW(s.handle, WM_HSCROLL, WPARAM((npos << u16::BITS | SB_THUMBTRACK.0) as usize), LPARAM(0)); }
                }
                return Some(LRESULT(0))
            }
            WM_MOUSEWHEEL => {
                if s.scr_v.nMax + 1 > s.scr_v.nPage as i32 {
                    let delta = (wparam.0 >> u16::BITS) as i16 / WHEEL_DELTA as i16;
                    let mut npos = s.scr_v.nPos - (delta * (s.scr_v.nPage / 10) as i16) as i32;
                    if npos < s.scr_v.nMin { npos = s.scr_v.nMin; }
                    if npos > s.scr_v.nMax - s.scr_v.nPage as i32 + 1 { npos = s.scr_v.nMax - s.scr_v.nPage as i32 + 1; }
                    unsafe { SendMessageW(s.handle, WM_VSCROLL, WPARAM((npos << u16::BITS | SB_THUMBTRACK.0) as usize), LPARAM(0)); }
                }
                return Some(LRESULT(0))
            }
            _ => { }
        }
        None
    }
}

extern "system" fn enum_child(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let (id, hwndreturn) = unsafe { &mut *(lparam.0 as *mut (isize, &mut HWND)) };
    if unsafe { GetDlgCtrlID(hwnd) } != *id as i32 { return TRUE }
    **hwndreturn = hwnd;
    FALSE
}

pub fn get_ctrl(hwnd_root: HWND, id: isize) -> HWND {
    let mut hwnd = HWND::default();
    unsafe { EnumChildWindows(hwnd_root, Some(enum_child), LPARAM(&mut (id, &mut hwnd) as *mut _ as _)); }
    hwnd
}

pub fn is_ctrl_enable(hwnd_root: HWND, id: isize) -> bool {
    unsafe { IsWindowEnabled(get_ctrl(hwnd_root, id)) }.into()
}

pub fn get_ctrl_checked(hwnd_root: HWND, id: isize) -> bool {
    BST_CHECKED.0 == unsafe { SendMessageW(get_ctrl(hwnd_root, id), BM_GETCHECK, WPARAM(0), LPARAM(0)) }.0 as u32
}

pub fn get_ctrl_text(hwnd_root: HWND, id: isize) -> String {
    let hctrl = get_ctrl(hwnd_root, id);
    let mut buf = [0u16; MAX_PATH as usize];
    let len = unsafe { GetWindowTextW(hctrl, &mut buf) } as usize;
    if len > 0 {
        WSTR::from_slice_to_string(&buf, len) // null終端無視
    } else {
        String::default()
    }
}

pub fn get_ctrl_int(hwnd_root: HWND, id: isize) -> isize {
    let text = get_ctrl_text(hwnd_root, id);
    if let Ok(i) = text.parse::<isize>() { i } else { 0 }
}

pub fn get_ctrl_cursel(hwnd_root: HWND, id: isize) -> isize {
    unsafe { SendMessageW(get_ctrl(hwnd_root, id), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 }
}

pub fn set_ctrl_enable(hwnd_root: HWND, id: isize, b_enable: bool) {
    unsafe { EnableWindow(get_ctrl(hwnd_root, id), if b_enable { TRUE } else { FALSE }); }
}

pub fn set_ctrl_checked(hwnd_root: HWND, id: isize, bchecked: bool) {
    unsafe { SendMessageW(get_ctrl(hwnd_root, id), BM_SETCHECK, WPARAM(if bchecked { BST_CHECKED.0 } else { BST_UNCHECKED.0 } as usize), LPARAM(0)); }
}

pub fn set_ctrl_text(hwnd_root: HWND, id: isize, text: &str) {
    let _ = unsafe { SetWindowTextW(get_ctrl(hwnd_root, id), WSTR::from(text).PCWSTR()) };
}

pub fn set_ctrl_int(hwnd_root: HWND, id: isize, int: isize) {
    set_ctrl_text(hwnd_root, id, &int.to_string());
}

pub fn set_ctrl_cursel(hwnd_root: HWND, id: isize, idx: usize) {
    unsafe { SendMessageW(get_ctrl(hwnd_root, id), CB_SETCURSEL, WPARAM(idx), LPARAM(0)); }
}

pub fn init_cont_hstack<T: 'static>(inst_subproc: Weak<T>,wc: &mut WindowContainerRc, w: i32, h: i32, a: WidthAuto, v: AlignV, cmdid: isize) -> WindowContainerRc
where Weak<T>: WindowContainerMsgProc {
    let mut c = wc.create_child_container(WS_EX_CONTROLPARENT, WS_VISIBLE, HMENU(cmdid)).upgrade().unwrap();
    c.set_msg_proc(Some(Box::new(inst_subproc)));
    wc.hstack(c.handle(), w, h, 0, 0, 0, true, a, v);
    c
}

pub fn init_cont_vstack<T: 'static>(inst_subproc: Weak<T>, wc: &mut WindowContainerRc, w: i32, h: i32, v: AlignH, a: HeightAuto, cmdid: isize)  -> WindowContainerRc
where Weak<T>: WindowContainerMsgProc {
    let mut c = wc.create_child_container(WS_EX_CONTROLPARENT, WS_VISIBLE, HMENU(cmdid)).upgrade().unwrap();
    c.set_msg_proc(Some(Box::new(inst_subproc)));
    wc.vstack(c.handle(), w, h, 0, 0, 0, true, v, a);
    c
}

pub fn init_cont_place<T: 'static>(inst_subproc: Weak<T>, wc: &mut WindowContainerRc, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind:  PlaceSet, span_kind: PlaceSet, cmdid: isize)  -> WindowContainerRc
where Weak<T>: WindowContainerMsgProc {
    let mut c = wc.create_child_container(WS_EX_CONTROLPARENT, WS_VISIBLE, HMENU(cmdid)).upgrade().unwrap();
    c.set_msg_proc(Some(Box::new(inst_subproc)));
    wc.place(c.handle(), x, y, w, h, pos_offset_hwnd, span_offset_hwnd, pos_kind, span_kind);
    c
}

pub fn init_item_hstack(wc: &mut WindowContainerRc, hfont: HFONT, w: i32, h: i32, a: WidthAuto, v: AlignV, clsname: &str, text: &str, style: WINDOW_STYLE, cmdid: isize) {
    let hctrl = wc.create_child(WINDOW_EX_STYLE::default(), WSTR::from(clsname).PCWSTR(), WSTR::from(text).PCWSTR(), WS_VISIBLE | style, HMENU(cmdid), true);
    unsafe { SendMessageW(hctrl,  WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(0/*FALSE*/)); }
    let (wf, hf) = adjust_ctrl_textsize(hctrl, hfont, w, h, text);
    wc.hstack(hctrl, wf, hf, 2, 0, 0, true, a, v);
}

pub fn init_item_vstack(wc: &mut WindowContainerRc, hfont: HFONT, w: i32, h: i32, v: AlignH, a: HeightAuto, clsname: &str, text: &str, style: WINDOW_STYLE, cmdid: isize) {
    let hctrl = wc.create_child(WINDOW_EX_STYLE::default(), WSTR::from(clsname).PCWSTR(), WSTR::from(text).PCWSTR(), WS_VISIBLE | style, HMENU(cmdid), true);
    unsafe { SendMessageW(hctrl,  WM_SETFONT, WPARAM(hfont.0 as usize), LPARAM(0/*FALSE*/)); }
    let (wf, hf) = adjust_ctrl_textsize(hctrl, hfont, w, h, text);
    wc.vstack(hctrl, wf, hf, 2, 0, 0, true, v, a);
}

pub fn init_item_place(wc: &mut WindowContainerRc, x: f64, y: f64, w: f64, h: f64, pos_offset_hwnd: HWND, span_offset_hwnd: HWND, pos_kind:  PlaceSet, span_kind: PlaceSet, clsname: &str, text: &str, style: WINDOW_STYLE, cmdid: isize) {
    let hctrl = wc.create_child(WINDOW_EX_STYLE::default(), WSTR::from(clsname).PCWSTR(), WSTR::from(text).PCWSTR(), WS_VISIBLE | style, HMENU(cmdid), true);
    wc.place(hctrl, x, y, w, h, pos_offset_hwnd, span_offset_hwnd, pos_kind, span_kind);
}

fn adjust_ctrl_textsize(hwnd: HWND, hfont: HFONT, w: i32, h: i32, text: &str) -> (i32, i32) {
    if w == -1 || h == -1 {
        let (wf, hf) = text_size(hwnd, hfont, text);
        (if w != -1 { w } else { wf }, if h != -1 { h } else { hf })
    } else {
        (w, h)
    }
}

pub fn text_size(hwnd: HWND, hfont: HFONT, text: &str) -> (i32, i32) {
    let mut rc = RECT::default();
    unsafe {
        let hdc = GetDC(hwnd);
        let oldobj = SelectObject(hdc, hfont);
        DrawTextW(hdc, &mut WSTR::from(text).0, &mut rc, DT_CALCRECT);
        SelectObject(hdc, oldobj);
    }
    (rc.right, rc.bottom)
}

pub fn sys_metrics(hwnd: HWND, metrics: SYSTEM_METRICS_INDEX) -> i32 {
    unsafe { GetSystemMetricsForDpi(metrics, GetDpiForWindow(hwnd)) }
}

pub fn sys_metrics_without_wnd(metrics: SYSTEM_METRICS_INDEX) -> i32 {
    let mut pt = POINT::default();
    let (mut dpix, mut dpiy) = (0u32, 0u32);
    unsafe {
        let _ = GetCursorPos(&mut pt);
        let h = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        if GetDpiForMonitor(h, MDT_EFFECTIVE_DPI, &mut dpix, &mut dpiy).is_err() { dpix = 96; }
        GetSystemMetricsForDpi(metrics, dpix)
    }
}

// 外周(Non Client領域)を含めたウィンドウのサイズを決定する
pub fn adjust_window_rect(hwnd: HWND, w_client: i32, h_client: i32) -> (i32 /*w_window*/, i32 /*h_window*/) {
    let dw_style = WINDOW_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32);
    let dw_exstyle = WINDOW_EX_STYLE(unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32);
    let mut rc = RECT { left: 0, top:0, right: w_client, bottom: h_client };
    let _ = unsafe { AdjustWindowRectExForDpi(&mut rc, dw_style, FALSE, dw_exstyle, GetDpiForWindow(hwnd)) };
    (rc.right - rc.left, rc.bottom - rc.top)
}

pub fn sys_font_init(hwnd: HWND) -> (Font, i32) {
    let ref mut ncm = NONCLIENTMETRICSW {
        cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };

    let _ = unsafe { SystemParametersInfoForDpi(SPI_GETNONCLIENTMETRICS.0, ncm.cbSize, Some(ncm as *mut _ as _), 0, GetDpiForWindow(hwnd)) };
    ncm.lfCaptionFont.lfWidth = 0;
    ncm.lfCaptionFont.lfHeight = (ncm.lfCaptionFont.lfHeight as f64 * 1.1) as _;
    let hfont = unsafe { CreateFontIndirectW(&ncm.lfCaptionFont) };

    let (_, height) = text_size(hwnd, hfont, CHAR_FONT_HEIGHT_MEASURE);
    (Font(hfont), height)
}
