use std::cell::UnsafeCell;

use fxhash::FxHashMap;

use windows::Win32::{
    UI::Shell::Common::ITEMIDLIST,
    System::{Ole::*, Com::{IDataObject, CLSCTX_INPROC_SERVER, CoCreateInstance, FORMATETC, DVASPECT_CONTENT, TYMED_HGLOBAL}, SystemServices::{SFGAO_FOLDER, MODIFIERKEYS_FLAGS, MK_LBUTTON, MK_RBUTTON}, Memory::{GlobalUnlock, GlobalLock, GlobalSize}, DataExchange::RegisterClipboardFormatW, },
};

use super::*;
use crate::lib_property::PropertyHolder;

#[derive(Default)]
pub struct ObjectHolder {
    pub parse_name: String,
    pub str_param: String,
    pub ishell_folder: Option<IShellFolder>,
    pub icontext_menu3: Option<IContextMenu3>,
    pub list_items: Vec<ObjectItem>,
}

pub struct ObjectItem {
    pub idl_rel: ItemIDList,
    pub attr: u32,
    pub str_disp_name: WSTR,
    pub str_parse_name: WSTR,
    pub icon_sm: Icon,
    pub icon_lr: Icon,
}

impl ObjectHolder {
    fn enumerate(isf: IShellFolder, parse_name: String, b_op_hidden: Option<bool>) -> Result<ObjectHolder> {
        let mut vec = Vec::<ObjectItem>::default();
        let mut vec_parsename = Vec::<String>::default();
        let mut map = FxHashMap::<String, ObjectItem>::default();

        let mut sortlist = PropertyHolder::load_filesort_param(&parse_name);
        let str_param = if sortlist.len() > 0 { sortlist.remove(0) } else { String::default() };

        let ienum_id_list: IEnumIDList = {
            let b_hidden = b_op_hidden.or_else(||Some(PropertyHolder::parse_string(&str_param).b_sysfile_hidden)).unwrap();
            let enumflag = (SHCONTF_FOLDERS.0 | SHCONTF_NONFOLDERS.0) | if !b_hidden { SHCONTF_INCLUDEHIDDEN.0 |SHCONTF_INCLUDESUPERHIDDEN.0 } else { 0 };

            let mut ptr =  0usize;
            unsafe { isf.EnumObjects(None, enumflag as u32, &mut ptr as *mut _ as _) }.ok()?;
            unsafe { std::mem::transmute(ptr) } // 実体化しないとRelease呼ばれない
        };

        let mut itemlist = [0usize as *mut ITEMIDLIST];
        let mut pceltfetched = 0u32;
        let mut result = unsafe { ienum_id_list.Next(&mut itemlist, Some(&mut pceltfetched)) };

        while result == S_OK {
            let pidl = ItemIDList(itemlist[0]); // auto drop resource(ObjectItemへ保存)

            let str_disp_name = Self::get_object_name(&isf, itemlist[0], SHGDN_NORMAL);
            let str_parse_name = Self::get_object_name(&isf, itemlist[0], SHGDNF(SHGDN_FORPARSING.0 | SHGDN_INFOLDER.0));

            let itemlistc = [itemlist[0] as *const ITEMIDLIST];
            let mut attr = SFGAO_FOLDER.0;
            let _ = unsafe { isf.GetAttributesOf(&itemlistc, &mut attr) };

            let (icon_l, icon_s) = Icon::load_file_icon(&isf, itemlist[0]); // auto drop resource(ObjectItemへ保存)

            let i = ObjectItem {
                idl_rel: pidl,
                attr: attr,
                str_disp_name: str_disp_name,
                str_parse_name: WSTR(str_parse_name.0.clone()), // MOVE回避のため複製
                icon_sm: icon_s,
                icon_lr: icon_l,
            };
            map.insert(str_parse_name.to_string_null_search(), i);
            vec_parsename.push(str_parse_name.to_string_null_search());

            result = unsafe { ienum_id_list.Next(&mut itemlist, Some(&mut pceltfetched)) };
        }

        if sortlist.len() > 0 { // ソート保存に従って並べる(ソート保存に値がないアイテムは後ろに回る)
            for i in sortlist.into_iter() {
                let v = map.remove(&i);
                if let Some(v) = v {
                    vec.push(v);
                }
            }
            for i in vec_parsename.iter() {
                let v = map.remove(i);
                if let Some(v) = v {
                    vec.push(v);
                }
            }
        } else { // ソート保存がない時はEnum順を維持するが、ディレクトリは先に並べる
            let mut v_file = Vec::<ObjectItem>::default();
            for i in vec_parsename.iter() {
                let v = map.remove(i);
                if let Some(v) = v {
                    if (v.attr & SFGAO_FOLDER.0 as u32) != 0 {
                        vec.push(v);
                    } else {
                        v_file.push(v);
                    }
                }
            }
            vec.append(&mut v_file);
        }

        if vec.len() == 0 { // フォルダが空の時
            vec.push(ObjectItem {
                idl_rel: ItemIDList(0usize as _),
                attr: 0u32,
                str_disp_name: WSTR::from(OBJECTITEM_EMPTY),
                str_parse_name: WSTR::from(""),
                icon_sm: Icon(HICON(0)),
                icon_lr: Icon(HICON(0)),
            })
        }

        Ok(ObjectHolder {
            parse_name,
            str_param: str_param,
            ishell_folder: Some(isf),
            icontext_menu3: None,
            list_items: vec,
        })
    }

    fn parse_object(path: &str) -> Result<(IShellFolder, String)> {
        let mut str_parse_name = String::new();
        let ishell_folder =
            if "".ne(path) {
                let p = WSTR::from(path);
                let mut eaten = 0u32;
                let mut itemlist: [*const ITEMIDLIST; 1] = [0usize as *const ITEMIDLIST];
                let mut attr = 0u32;
                let desktop: IShellFolder = unsafe { SHGetDesktopFolder() }?;
                unsafe { desktop.ParseDisplayName(None, None, p.PCWSTR(), Some(&mut eaten),
                    &mut itemlist[0] as *mut *const ITEMIDLIST as _, &mut attr)?; }
                let _pidl = ItemIDList(itemlist[0] as _); // auto drop resource

                str_parse_name = Self::get_object_name(&desktop, itemlist[0], SHGDN_FORPARSING).to_string_null_search();

                unsafe { desktop.BindToObject(itemlist[0] as *const ITEMIDLIST, None) }?
            } else {
                unsafe { SHGetDesktopFolder() }?
            };

        Ok((ishell_folder, str_parse_name))
    }

    pub fn bind(path: &str, b_hidden: bool) -> Result<ObjectHolder> {
        let (ishell_folder, str_parse_name) = Self::parse_object(path)?;
        Self::enumerate(ishell_folder, str_parse_name, Some(b_hidden))
    }

    pub fn child(&mut self, idx: usize) -> Result<ObjectHolder> {
        let ishell_child: IShellFolder =
            unsafe { self.ishell_folder.as_mut().unwrap().BindToObject(self.list_items[idx].idl_rel.0, None) }?;

        let str_parse_name =
            Self::get_object_name(&self.ishell_folder.as_ref().unwrap(), self.list_items[idx].idl_rel.0, SHGDN_FORPARSING)
            .to_string_null_search();

        Self::enumerate(ishell_child, str_parse_name, None)
    }

    pub fn get_object_name(isf: &IShellFolder, pidl: *const Common::ITEMIDLIST, uflags: SHGDNF) -> WSTR {
        (|| {
            let mut buf = [0u16; MAX_PATH as usize];
            let mut strret = Common::STRRET::default();
            unsafe { isf.GetDisplayNameOf(pidl, uflags, &mut strret) }.ok()?;
            unsafe { StrRetToBufW(&mut strret, Some(pidl), &mut buf) }.ok()?;
            Some(WSTR::from_slice_null_search(&buf))
        })().or(Some(WSTR::new(1))).unwrap()
    }

    pub fn set_object_name(path: &str, oldname: &str, newname: &str) -> Result<()> {
        let (ishell_folder, _) = Self::parse_object(path)?;

        let mut eaten = 0u32;
        let mut itemlist: [*const ITEMIDLIST; 1] = [0usize as *const ITEMIDLIST];
        let mut attr = 0u32;
        unsafe {
            ishell_folder.ParseDisplayName(None, None, WSTR::from(oldname).PCWSTR(), Some(&mut eaten),
            &mut itemlist[0] as *mut *const ITEMIDLIST as _, &mut attr)?;
            let mut _pidl = ItemIDList(itemlist[0] as _); // auto drop resource
            ishell_folder.SetNameOf(None, itemlist[0], WSTR::from(newname).PCWSTR(), SHGDN_NORMAL, None)?;
        }
        Ok(())
    }

    pub fn get_ui_object_of<T>(&self, handle: HWND, idx: i32) -> Result<T>
    where T: windows::core::ComInterface {
        if idx < 0 {
            // IContextMenuやDataObjectなどのGetUIObjectOf取得には、対象直属の親のIShellFolder::GetUIObjectOfに子の相対IDL(単一)を渡さないといけない。
            // ということで、self.ishell_folder自身のオブジェクト取得は面倒だけどこうなる。
            let me_full = ItemIDList(unsafe { SHGetIDListFromObject(self.ishell_folder.as_ref().unwrap()) }?);
            let pa = ItemIDList(unsafe { ILClone(me_full.0) });

            unsafe { ILRemoveLastID(Some(pa.0)); } // 最終IDを削除する＝直属の親の絶対IDLになる
            let me = unsafe { ILFindLastID(me_full.0) }; // meはme_fullの最終位置の参照なので開放不要

            let parent: IShellFolder = unsafe { SHBindToObject (None, pa.0, None) }?;
            unsafe { parent.GetUIObjectOf(handle, &[me], None) }
        } else {
            if self.list_items[idx as usize].idl_rel.0.is_null() { return Err(Error::OK) }
            unsafe { self.ishell_folder.as_ref().unwrap().GetUIObjectOf(handle, &[self.list_items[idx as usize].idl_rel.0], None) }
        }
    }

    pub fn do_menu(&mut self, idx: i32, handle: HWND, x: i32, y: i32, b_extend: bool, b_popup: bool) -> Result<()> {
        let icm = self.get_ui_object_of::<IContextMenu>(handle, idx)?;
        let icm3 = icm.cast::<IContextMenu3>()?;

        let hmenu = Menu(unsafe { CreatePopupMenu() }?); // auto drop resource

        let (index, offset) = if idx < 0 {
            let mut mii = MENUITEMINFOW::default();
            mii.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
            mii.fMask = MIIM_ID | MIIM_STRING;
            mii.fType = MFT_STRING;
            mii.wID = 1;
            mii.dwTypeData = PWSTR::from_raw(POPUP_MENUITEM_PROP.as_ptr() as _);
            unsafe { InsertMenuItemW(hmenu.0, 0, TRUE, &mii)?; }

            mii.wID = 2;
            mii.dwTypeData = PWSTR::from_raw(POPUP_MENUITEM_SORT_RESET.as_ptr() as _);
            unsafe { InsertMenuItemW(hmenu.0, 1, TRUE, &mii)?; }

            mii.fMask = MIIM_FTYPE;
            mii.fType = MFT_SEPARATOR;
            unsafe { InsertMenuItemW(hmenu.0, 2, TRUE, &mii)?; }

            (3, 3)
        } else {
            (0u32, 1i32)
        };

        unsafe { icm3.QueryContextMenu(hmenu.0, index, offset as u32, 0xffff, CMF_NORMAL | if b_extend { CMF_EXTENDEDVERBS } else { 0 } | CMF_EXPLORE |CMF_CANRENAME)? };
        let cmd: i32 =
            if b_popup {
                self.icontext_menu3 = Some(icm3.clone()); // ポップアップメニュー表示時のハンドラ用
                let r = unsafe { TrackPopupMenu(hmenu.0, TPM_LEFTALIGN | TPM_RETURNCMD, x, y, 0, handle, None).0 as i32};
                self.icontext_menu3 = None;
                r
            } else {
                unsafe { GetMenuDefaultItem(hmenu.0, FALSE.0 as u32, GMDI_USEDISABLED) as i32 }
            };

        if cmd == 0 {
            return Err(Error::OK)
        }
        if idx < 0 && cmd == 1 {
            return Err(Error::new(HRESULT(WMU_DIR_PROPERTY as i32), HSTRING::default()))
        }
        if idx < 0 && cmd == 2 {
            return Err(Error::new(HRESULT(WMU_DIR_SORT_RESET as i32), HSTRING::default()))
        }

        let mut buf = [0u16; MAX_PATH as usize];
        if unsafe { icm3.GetCommandString((cmd - offset) as usize, GCS_VERBW, None, PSTR::from_raw(&mut buf as *mut _ as _), MAX_PATH) }.is_ok() {
            if WSTR::from_slice_to_string_null_search(&buf).eq("rename") {
                return Err(Error::new(HRESULT(WMU_FILE_RENAME as i32), HSTRING::default()))
            }
        }

        let ici = CMINVOKECOMMANDINFOEX {
            cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: SEE_MASK_ASYNCOK | SEE_MASK_UNICODE | CMIC_MASK_PTINVOKE | if unsafe { GetAsyncKeyState(VK_SHIFT.0 as _) } as u16 & 0x8000 != 0 { CMIC_MASK_SHIFT_DOWN } else { 0 },
            hwnd: HWND(0),
            lpVerb: PCSTR::from_raw(((cmd - offset) as u16) as _), //#define MAKEINTRESOURCE(i) (LPTSTR) ((DWORD) ((WORD) (i)))
            lpVerbW: PCWSTR::from_raw(((cmd - offset) as u16) as _), //#define MAKEINTRESOURCE(i) (LPTSTR) ((DWORD) ((WORD) (i)))
            nShow: SW_SHOWNORMAL.0 as i32,
            ptInvoke: POINT { x: x, y: y },
            ..Default::default()
        };
        unsafe { icm3.InvokeCommand(&ici as *const _ as _) }
    }

    pub fn do_menu_handle(&mut self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Result<()> {
        if let Some(cm3) = &self.icontext_menu3 {
            unsafe { cm3.HandleMenuMsg2(umsg, wparam, lparam, None) }?;
        }
        Ok(())
    }
}

pub trait DropTargetWindow {
    fn get_handle(&self) -> HWND;
    fn get_droptarget(&mut self, mx: i32, my: i32, b_enter: bool) -> (Result<()>, Option<IDropTarget>);
}

#[implement(IDropTarget)]
pub struct MyDropTarget {
    parent: Box<dyn DropTargetWindow>,
    idth: Option<IDropTargetHelper>,
    idt: Option<IDropTarget>,
    ido: Option<IDataObject>,
    _unfreeze_mark: UnsafeCell<()>,
}

impl IDropTarget_Impl for MyDropTarget {
    #[allow(non_snake_case)]
    fn DragEnter(&self, pdataobj: Option<&IDataObject>, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> Result<()> {
        #[allow(invalid_reference_casting)]
        let s =  unsafe { &mut *(self as *const _ as *mut Self) };

        let mut point = POINT { x: pt.x, y: pt.y };
        unsafe { ScreenToClient(self.parent.get_handle(), &mut point); }

        let (r, v) = s.parent.get_droptarget(point.x, point.y, true);

        if r.is_err() || v.is_none() {
            s.idt = None;
            unsafe { *pdweffect = DROPEFFECT_NONE };
        } else {
            s.idt = v;
        }

        s.ido = Some(pdataobj.unwrap().clone()); // AddRef (s.idoのDropでReleaseが呼ばれる)

        let idth: IDropTargetHelper  = unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }?;
        s.idth = Some(idth);

        if s.idt.is_some() {
            unsafe { s.idt.as_mut().unwrap().DragEnter(pdataobj, grfkeystate, *pt, pdweffect) }?;
        }
        unsafe { s.idth.as_ref().unwrap().DragEnter(self.parent.get_handle(), pdataobj, pt as *const _ as *const POINT, *pdweffect) }
    }

    #[allow(non_snake_case)]
    fn DragOver(&self, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> Result<()> {
        #[allow(invalid_reference_casting)]
        let s =  unsafe { &mut *(self as *const _ as *mut Self) };

        let mut point = POINT { x: pt.x, y: pt.y };
        unsafe { ScreenToClient(self.parent.get_handle(), &mut point); }
        let (r, v) = s.parent.get_droptarget(point.x, point.y, false);

        if r.is_ok() {
            if  s.idt.is_some() {
                let _ = unsafe { s.idt.as_ref().unwrap().DragLeave() };
            }
            if v.is_some() {
                s.idt = v;
                unsafe { s.idt.as_ref().unwrap().DragEnter(Some(s.ido.as_ref().unwrap()), grfkeystate, *pt, pdweffect) }?;
            } else {
                s.idt = None;
            }
        }

       if s.idt.is_some() {
            unsafe { s.idt.as_ref().unwrap().DragOver(grfkeystate, *pt, pdweffect) }?;
        } else {
            unsafe { *pdweffect = DROPEFFECT_NONE };
        }
        unsafe { s.idth.as_ref().unwrap().DragOver(pt as *const _ as *const POINT, *pdweffect) }
    }

    #[allow(non_snake_case)]
    fn DragLeave(&self) -> Result<()> {
        #[allow(invalid_reference_casting)]
        let s =  unsafe { &mut *(self as *const _ as *mut Self) };

        if s.idt.is_some() {
            let _ = unsafe { s.idt.as_ref().unwrap().DragLeave() };
        }
        let _ = unsafe { s.idth.as_ref().unwrap().DragLeave() };

        s.ido = None;
        s.idt = None;
        s.idth = None;
        Ok(())
    }

    #[allow(non_snake_case)]
    fn Drop(&self, pdataobj: Option<&IDataObject>, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> Result<()> {
        #[allow(invalid_reference_casting)]
        let s =  unsafe { &mut *(self as *const _ as *mut Self) };

        if s.idt.is_some() {
            let _ = unsafe { s.idt.as_mut().unwrap().Drop(pdataobj, grfkeystate, *pt, pdweffect) };
        }
        let _ = unsafe { s.idth.as_ref().unwrap().Drop(pdataobj, pt as *const _ as *const POINT, *pdweffect) };

        s.ido = None;
        s.idt = None;
        s.idth = None;
        Ok(())
    }
}

pub struct MyDropTargetHolder {
    ptr_vtable: usize,
    impl_instance: MyDropTarget_Impl,
}

impl MyDropTargetHolder {
    pub fn new(parent: Box<dyn DropTargetWindow>) -> Box<Self> {
        Box::new(Self {
            ptr_vtable: 0,
            impl_instance: MyDropTarget_Impl::new(MyDropTarget {
                parent: parent,
                ido: None,
                idth: None,
                idt: None,
                _unfreeze_mark: UnsafeCell::default(),
            }),
        })
    }

    pub fn get_impl(&self) -> &MyDropTarget {
        self.impl_instance.get_impl()
    }

    pub fn regist(&mut self, hwnd: HWND) -> Result<()> {
        self.ptr_vtable = &self.impl_instance.vtables as *const _ as _;
        unsafe { RegisterDragDrop(hwnd, &*(&self.ptr_vtable as *const _ as *const IDropTarget)) }
    }

    pub fn unregist(&self, hwnd: HWND) -> Result<()> {
        if self.get_impl().idth.is_some() {
            let _ = unsafe { &self.get_impl().idth.as_ref().unwrap().DragLeave() };
        }
        unsafe { RevokeDragDrop(hwnd) }
    }
}

#[implement(IDropSource)]
pub struct MyDropSource {
    dobj: IDataObject,
    b_set_cursor: bool,
    _unfreeze_mark: UnsafeCell<()>,
}

impl MyDropSource {
    pub fn drag_and_drop(dobj: IDataObject) -> Result<HRESULT> {
        let idsh: IDragSourceHelper = unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }?;
        let idsh2: IDragSourceHelper2 = idsh.cast()?;
        unsafe { idsh2.SetFlags(DSH_ALLOWDROPDESCRIPTIONTEXT.0 as u32) }?;
        unsafe { idsh2.InitializeFromWindow(None, None, &dobj) }?;

        let i = MyDropSource_Impl::new(Self { dobj: dobj, b_set_cursor: true, _unfreeze_mark: UnsafeCell::default() } );
        let ptr_vtbl = &i.vtables as *const _;
        let mut dw_effect = DROPEFFECT::default();
        Ok(unsafe { DoDragDrop(&i.this.dobj, &*(&ptr_vtbl as *const _ as *const IDropSource),
            DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK, &mut dw_effect) })
    }
}

impl IDropSource_Impl for MyDropSource {
    #[allow(non_snake_case)]
    fn QueryContinueDrag(&self, fescapepressed: BOOL, grfkeystate: MODIFIERKEYS_FLAGS) -> HRESULT {
        if fescapepressed.into() { return DRAGDROP_S_CANCEL } // ESCキーでDDキャンセル
        if (grfkeystate.0 & (MK_LBUTTON.0 | MK_RBUTTON.0)) == 0u32 { return DRAGDROP_S_DROP } // マウスボタンリリースでDD実行
        S_OK
    }

    #[allow(non_snake_case)]
    fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
        #[allow(invalid_reference_casting)]
        let s =  unsafe { &mut *(self as *const _ as *mut Self) };

        let b_showing_layered = get_global_data_dword(&s.dobj, "IsShowingLayered");
        if b_showing_layered.is_err() { return DRAGDROP_S_USEDEFAULTCURSORS }

        if b_showing_layered.unwrap() != 0 {
            if s.b_set_cursor {
                let hcursor = unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap();
                unsafe { SetCursor(hcursor); }
                s.b_set_cursor = false;
            }
            let dw = get_global_data_dword(&s.dobj, "DragWindow");
            if dw.is_err() || dw.as_ref().unwrap() == &0 { return DRAGDROP_S_USEDEFAULTCURSORS }
            unsafe { SendMessageW(HWND(dw.unwrap() as isize), WM_USER + 3, WPARAM(0), LPARAM(0)); }
            S_OK

        } else {
            s.b_set_cursor = true;
            DRAGDROP_S_USEDEFAULTCURSORS
        }
    }
}

fn get_global_data_dword(dobj: &IDataObject, str_format: &str) -> Result<u32> {
    let mut fe = FORMATETC::default();
    fe.cfFormat = unsafe { RegisterClipboardFormatW(WSTR::from(str_format).PCWSTR()) } as u16;
    fe.dwAspect = DVASPECT_CONTENT.0;
    fe.lindex = -1;
    fe.tymed = TYMED_HGLOBAL.0 as u32;

    unsafe { dobj.QueryGetData(&fe as *const _ )}.ok()?;
    let mut sm = unsafe { dobj.GetData(&fe as *const _) }?;
    debug_assert!(unsafe { GlobalSize(sm.u.hGlobal) } >=  std::mem::size_of::<u32>());

    let dw = unsafe { *(GlobalLock(sm.u.hGlobal) as *mut u32) };
    unsafe { GlobalUnlock(sm.u.hGlobal) }?;
    unsafe { ReleaseStgMedium(&mut sm); }
	Ok(dw)
}
