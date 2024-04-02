use std::collections::VecDeque;
use fxhash::{FxHashMap, FxHashSet};
use windows::Win32::{
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED}, Storage::{EnhancedStorage::PKEY_AppUserModel_ID, FileSystem::FILE_FLAGS_AND_ATTRIBUTES},
    System::{Com::StructuredStorage::PropVariantClear, Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION}},
    UI::Shell::{Common::ITEMIDLIST, PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow}}
};

use super::*;
use self::{lib_gui_layout_container::sys_metrics_without_wnd, lib_property::PropertyHolder};

const WINLIST_IGNORE: [&str; 2] = ["Progman", "Internet Explorer_Hidden"];

impl Icon {
    fn get_uwp_icon(hwnd: HWND) -> Result<isize> {
        let i_property_store: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(hwnd) }?;
        let mut pv = unsafe { i_property_store.GetValue(&PKEY_AppUserModel_ID) }?;
        let mut str = WSTR::from_slice_null_search(unsafe { pv.Anonymous.Anonymous.Anonymous.bstrVal.as_wide() });
        unsafe { PropVariantClear(&mut pv) }?;

        if str.0.len() > 1 {
            let mut buf2 = WSTR::from(r"shell:AppsFolder\");
            buf2.0.pop(); // NULL終端をとる
            buf2.0.append(&mut str.0);
            Ok(Self::get_shell_file_icon(buf2.PCWSTR(), -1))
        } else {
            Err(Error::OK)
        }
    }

    // index:-1 => file icon
    pub fn get_shell_file_icon(path: PCWSTR, index: i32) -> isize {
        if index == -1 {
            let pidl = ItemIDList(unsafe { ILCreateFromPathW(path) }); // auto drop resource
            let mut si = SHFILEINFOW::default();
            unsafe { SHGetFileInfoW(PCWSTR::from_raw(pidl.0 as _), FILE_FLAGS_AND_ATTRIBUTES(0), Some(&mut si),
                std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_PIDL | SHGFI_ICON | SHGFI_SMALLICON) };
            si.hIcon.0
        } else {
            let mut h = HICON::default();
            unsafe { ExtractIconExW(path, index, None, Some(&mut h), 1); }
            h.0
        }
    }

    pub fn load_win_icon(hwnd: HWND, proc_img_fname: &str) -> Icon {
        let mut h = 0isize;

        if hwnd.0 != 0 {
            unsafe { SendMessageTimeoutW(hwnd, WM_GETICON, WPARAM(2), LPARAM(0), SMTO_ABORTIFHUNG | SMTO_BLOCK, 500, Some(&mut h as *mut _ as _)) };
            if h == 0 {
                unsafe { SendMessageTimeoutW(hwnd, WM_GETICON, WPARAM(0), LPARAM(0), SMTO_ABORTIFHUNG | SMTO_BLOCK, 500, Some(&mut h as *mut _ as _)) };
            }
            if h == 0 {
                h = Self::get_uwp_icon(hwnd).unwrap_or(0);
            }
        }
        if h == 0 {
            h = Self::get_shell_file_icon(WSTR::from(proc_img_fname).PCWSTR(), 0);
        }
        if h == 0 && hwnd.0 != 0 {
            h = unsafe { GetClassLongPtrW(hwnd, GCLP_HICONSM) } as isize;
        }
        if h == 0 {
            let mut sii = SHSTOCKICONINFO::default();
            sii.cbSize = std::mem::size_of::<SHSTOCKICONINFO>() as u32;
            let _ = unsafe { SHGetStockIconInfo(SIID_APPLICATION , SHGSI_ICON | SHGSI_SMALLICON, &mut sii) };
            h = sii.hIcon.0;
        }

        Icon(HICON(h))
    }

    pub fn load_file_icon(isf: &IShellFolder, pidl: *mut ITEMIDLIST) -> (Icon, Icon) {
        let mut h_icon_lr = HICON(0);
        let mut h_icon_sm = HICON(0);

        let _: Result<()> = (|| -> Result<()> {
            let mut buf = [0u16; MAX_PATH as usize];
            let mut index = 0i32;
            let mut flags = 0u32;
            let itemlistc = [pidl as *const ITEMIDLIST];

            let iext_icon:IExtractIconW = unsafe { isf.GetUIObjectOf(None, &itemlistc, None) }?;
            let (w_lr, w_sm) =  (sys_metrics_without_wnd(SM_CXICON), sys_metrics_without_wnd(SM_CXSMICON));
            unsafe {iext_icon.GetIconLocation(0, &mut buf, &mut index, &mut flags)}?;
            unsafe {iext_icon.Extract(PCWSTR::from_raw(&buf as _), index as u32, Some(&mut h_icon_lr), Some(&mut h_icon_sm), (w_sm << u16::BITS | w_lr) as u32)}?;
            if h_icon_lr.0 != 0isize { Ok(()) } else { Err(Error::OK) }

        })().or_else(|_| {
            let mut sii = SHSTOCKICONINFO::default();
            sii.cbSize = std::mem::size_of::<SHSTOCKICONINFO>() as u32;
            let _ = unsafe { SHGetStockIconInfo(SIID_APPLICATION , SHGSI_ICON | SHGSI_SMALLICON, &mut sii) }?;
            h_icon_sm = sii.hIcon;
            unsafe { SHGetStockIconInfo(SIID_APPLICATION , SHGSI_ICON | SHGSI_LARGEICON, &mut sii) }?;
            h_icon_lr = sii.hIcon;
            Ok(())
        });

        (Icon(h_icon_lr), Icon(h_icon_sm))
    }
}

#[derive(Clone)]
pub struct WindowInfo {
    pub group_type: u32, // 0-プロセス自動・ソート固定、1-プロセス自動・ソート非固定、2-テンポラリ
    pub handle: HWND,
    pub proc_img_fname: String,
    pub title: String,
}

impl WindowInfo {
    pub fn sort_window_list(sortlist: &Vec<WinSortList>, nowlist: Vec<WindowInfo>, wingrplist: &mut VecDeque<VecDeque<WindowInfo>>) {

        let mut map_hwnd = FxHashMap::<isize/*HWND*/, usize/*grpidx*/>::default();
        let mut map_procimg = FxHashMap::<String, usize/*grpidx*/>::default();

        // 元のグループリストに含まれるウィンドウを列挙(HWNDで引き出せるようにmap準備。procimgをキーに既存グループをgrpidxで引き出せるようmap準備)
        for (grpidx, v) in wingrplist.iter().enumerate() {
            for (itemidx, i) in v.iter().enumerate() {
                if itemidx == 0 && (i.group_type == 0 || i.group_type == 1) {
                    map_procimg.insert(i.proc_img_fname.clone(), grpidx);
                }
                map_hwnd.insert(i.handle.0, grpidx);
            }
        }

        // 新ウィンドウ一覧（Ａ）と、元グループリスト（Ｂ）をHWNDで突合し、マッチしたらtitleを更新、アンマッチなら新winをグループリストに追加
        for mut win in nowlist.into_iter() {
            let values = map_hwnd.remove(&win.handle.0);
            let mut b = true;
            if let Some(grpidx) = values { // マッチ
                for i in wingrplist[grpidx].iter_mut() { // vecをなめてhwnd一致するものを捜索
                    if i.handle == win.handle {
                        if i.proc_img_fname.ne(&win.proc_img_fname) { // 別プロセスの同じウィンドウハンドルが出現
                            map_hwnd.insert(win.handle.0, grpidx); // 旧は削除対象にマーク。b=falseとしないことでwinには新規追加
                        } else {
                            i.title = win.title.clone(); // タイトル更新
                            b = false;
                        }
                        break;
                    }
                }
            }
            if b { // 新規ウィンドウ（Ａ）オンリー
                let vec_group = map_procimg.get(&win.proc_img_fname);
                if vec_group.is_none() {
                    let mut vec_group = VecDeque::<WindowInfo>::default();
                    let k = win.proc_img_fname.clone();
                    win.group_type = u32::MAX;
                    vec_group.push_front(win);
                    wingrplist.push_back(vec_group); // grpidxを壊さないよう一旦末尾に追加し後でソート★
                    map_procimg.insert(k, wingrplist.len() - 1);
                } else {
                    win.group_type = wingrplist[*vec_group.unwrap()][0].group_type;
                    wingrplist[*vec_group.unwrap()].push_front(win);
                }
            }
        }

        // 元グループリスト側（Ｂ）オンリーのHNWDは、消滅したウィンドウに該当するのでリストから削除
        for (hwnd, grpidx) in map_hwnd { // なくなったウィンドウを削除
            for (idx, i) in wingrplist[grpidx].iter_mut().enumerate() { // vecをなめてhwnd一致するものを捜索
                if i.handle.0 == hwnd {
                    wingrplist[grpidx].remove(idx); // グループ内のウィンドウを削除
                    break;
                }
            }
        }
        for idx in (0..wingrplist.len()).rev() { // 空のグループを削除。要素を消したときにidxがずれるので後ろから辿る
            if wingrplist[idx].len() == 0 {
                wingrplist.remove(idx);
            }
        }

        // ★の処理
        let mut vecgrp = VecDeque::<isize>::default(); // ソートのスコアを格納
        for grpidx in 0..wingrplist.len() {
            let maxidx = wingrplist[grpidx].len() - 1;

            // scoreチェックの前に、group_typeをチェックして0でないならscoreは-1とする。
            if wingrplist[grpidx][maxidx].group_type == 1 || wingrplist[grpidx][maxidx].group_type == 2 {
                vecgrp.push_back(-1);
                continue;
            }

            // sortlistを探索してscoreを決定（sortlistパラメータにない場合は-1）
            let score =
                match PropertyHolder::contains_window_sort_list(&sortlist, &wingrplist[grpidx][maxidx].proc_img_fname) {
                    None => -1isize,
                    Some(s) => s as _,
                };

            if wingrplist[grpidx][maxidx].group_type == 0 { // group_type=0なら既存グループ
                vecgrp.push_back(score);

            } else { // 新規グループの追加
                if score == -1 { // score == -1 → sortlistパラメータにないグループは、先頭にグループ追加
                    for tmpidx in 0..=maxidx {
                        wingrplist[grpidx][tmpidx].group_type = 1;
                    }

                    vecgrp.push_front(score);
                    let grp = wingrplist.remove(grpidx).unwrap();
                    wingrplist.push_front(grp);

                } else {
                    for tmpidx in 0..=maxidx {
                        wingrplist[grpidx][tmpidx].group_type = 0;
                    }

                    // ソート実行
                    let mut b_insert = false;
                    for sortidx in 0..vecgrp.len() {
                        if vecgrp[sortidx] > score {
                            vecgrp.insert(sortidx, score);
                            let grp = wingrplist.remove(grpidx).unwrap();
                            wingrplist.insert(sortidx, grp);
                            b_insert = true;
                            break;
                        }
                    }
                    if !b_insert {
                        vecgrp.push_back(score);
                        // 末尾までいったということなのでremove,insertは冗長(wingrplistの末尾に既に存在している)
                        // let grp = wingrplist.remove(grpidx).unwrap();
                        // wingrplist.insert(0, grp);
                    }
                }
            }
        }
    }

    pub fn merge_proc_list(vec_wi: &Vec<WindowInfo>, sort_list: &Vec<String>, candidate_list: &mut Vec<String>) -> FxHashMap<String, HWND> {
        let mut sl_map = FxHashSet::<&String>::default();
        let mut cl_map = FxHashSet::<String>::default();
        let mut ret  = FxHashMap::<String, HWND>::default();

        for l in sort_list { sl_map.insert(l); }
        for l in &*candidate_list { cl_map.insert(l.clone()); }

        for wi in vec_wi {
            let r = PropertyHolder::contains_procimg_list(&mut sl_map.iter(), &wi.proc_img_fname);
            if r.is_some() {
                if r.as_ref().unwrap().contains("*") { ret.insert(r.unwrap(), wi.handle); }
            } else if !cl_map.contains(&wi.proc_img_fname) {
                cl_map.insert(wi.proc_img_fname.clone());
                candidate_list.push(wi.proc_img_fname.clone());
            }
        }
        ret
    }

    fn enum_window_base(hwnd: HWND) -> Option<WindowInfo> {
        let mut buf = [0u16; 512];

        let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
        let text = WSTR::from_slice_to_string(&buf, len as usize);
        if text.is_empty() { return None }

        let mut b: BOOL = false.into();
        let r = unsafe {DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED,
            &mut b as *mut _ as _, std::mem::size_of::<BOOL>() as u32) };
        if r.is_ok() && b.into() { return None }

        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        let class = WSTR::from_slice_to_string(&buf, len as usize);
        for title_ignore in WINLIST_IGNORE {
            if class.eq(title_ignore) { return None }
        }

        let mut pid = 0u32;
        let r = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if r == 0 { return None }

        let hp = Handle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) }.ok()?); // auto drop resource

        let mut len = buf.len() as u32;
        unsafe { QueryFullProcessImageNameW(hp.0, PROCESS_NAME_FORMAT(0), PWSTR::from_raw(&mut buf as _), &mut len) }.ok()?;

        let proc_img_fname = WSTR::from_slice_to_string(&buf, len as usize);
        if proc_img_fname.is_empty() { return None }

        Some(WindowInfo { group_type: u32::MAX, handle: hwnd, proc_img_fname: proc_img_fname, title: text })
    }

    pub extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !unsafe { IsWindowVisible(hwnd).into() } || !unsafe { IsWindowEnabled(hwnd).into() } {
            return TRUE
        }

        let r = Self::enum_window_base(hwnd);
        if let Some(i) = r {
            let v =  unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };
            v.push(i);
        }
        TRUE
    }

    pub extern "system" fn enum_window_mine(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let (mine, myclass) =  unsafe { &mut *(lparam.0 as *mut (&mut Result<HWND>, &String)) };
        let mut buf = [0u16; 32];
        let ret_len = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
        if ret_len > 0 {
            let classname = WSTR::from_slice_to_string(&buf, ret_len);
            if classname.eq(&(**myclass)) {
                **mine = Ok(hwnd);
                return FALSE
            }
        }
        TRUE
    }
}