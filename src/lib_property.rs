use windows::Win32::System::Registry::*;

use super::*;

const REGKEY_APP_PARAM: &str = if cfg!(debug_assertions) {
    r"SOFTWARE\myprogram\mytaskbar_D"
} else {
    r"SOFTWARE\myprogram\mytaskbar"
};
const REGKEY_FILE_LIST_ORDER: &str = if cfg!(debug_assertions) {
    r"SOFTWARE\myprogram\mytaskbar_flo_D"
} else {
    r"SOFTWARE\myprogram\mytaskbar_flo"
};

const REG_NOTIFY_ICON: &str = "notify_icon";
const REG_WIN_SORT: &str = "win_sort";

const HOTKEY_PARAM_TASK: &str = "TASK";
const HOTKEY_PARAM_FILELIST: &str = "LIST";
const HOTKEY_PARAM_FILEICON: &str = "ICON";
const HOTKEY_PARAM_ICON_SM: &str = "SM";
const HOTKEY_PARAM_ICON_LG: &str = "LG";
const HOTKEY_PARAM_SYSHIDE_T: &str = "HIDE";
const HOTKEY_PARAM_SYSHIDE_F: &str = "SHOW";
const HKMOD_CHAR_ALT: char = 'A';
const HKMOD_CHAR_SHIFT: char = 'S';
const HKMOD_CHAR_NONE: char = 'N';

#[derive(Default, PartialEq)]
pub enum HotkeyType { IconLauncher, #[default] ListLauncher, WinTaskList, }

pub struct PropertyHolder {
    pub hotkey_type: HotkeyType,
    pub b_icon_large: bool,
    pub w: u32,
    pub h: u32,
    pub b_sysfile_hidden: bool,
    pub path: String,
}

impl Default for PropertyHolder {
    fn default() -> Self {
        Self {
            hotkey_type: HotkeyType::default(),
            b_icon_large: bool::default(),
            w: u32::default(),
            h: u32::default(),
            b_sysfile_hidden: true,
            path: String::default(),
        }
    }
}

impl PropertyHolder {
    pub fn new(hotkey_type: HotkeyType, b_icon_large: bool, w: u32, h: u32, b_sysfile_hidden: bool, path: String) -> Self {
        Self {
            hotkey_type: hotkey_type,
            b_icon_large: b_icon_large,
            w: w,
            h: h,
            b_sysfile_hidden: b_sysfile_hidden,
            path: path,
        }
    }

    pub fn parse_string(l: &str) -> Self {
        let p: Vec<&str> = l.split(',').collect();

        // ^T,([0-9]+),([0-9]+),$
        if let Some(v) = (|| {
            if p.len() != 4 { return None }
            if !p[0].eq(HOTKEY_PARAM_TASK) { return None }
            let w = p[1].parse::<u32>().ok()?;
            let h = p[2].parse::<u32>().ok()?;
            Some(Self { hotkey_type: HotkeyType::WinTaskList, w: w, h: h, ..Default::default() })
        })() { return v };

        // ^L,([0-9]+),([0-9]+),(T|F)(,(.*)|$)
        if let Some(v) = (|| {
            if p.len() < 4 { return None }
            if !p[0].eq(HOTKEY_PARAM_FILELIST) { return None }
            let w = p[1].parse::<u32>().ok()?;
            let h = p[2].parse::<u32>().ok()?;
            let b_syshide =
                if p[3].eq(HOTKEY_PARAM_SYSHIDE_T) { true }
                else if p[3].eq(HOTKEY_PARAM_SYSHIDE_F) { false }
                else { return None };
            let mut path = String::new();
            if p.len() > 4 { // pathの中にカンマを含んでいる場合の対処
                for i in 4..=p.len() - 1 { path += p[i]; }
            }
            Some(Self { hotkey_type: HotkeyType::ListLauncher, w: w, h: h, b_sysfile_hidden: b_syshide, path: path, ..Default::default() })
        })() { return v };

        // ^I,(S|L),([0-9]+),([0-9]+),(T|F)(,(.*)|$)
        if let Some(v) = (|| {
            if p.len() < 5 { return None }
            if !p[0].eq(HOTKEY_PARAM_FILEICON) { return None }
            let b_large =
                if p[1].eq(HOTKEY_PARAM_ICON_LG) { true }
                else if p[1].eq(HOTKEY_PARAM_ICON_SM) { false }
                else { return None };
            let w = p[2].parse::<u32>().ok()?;
            let h =  p[3].parse::<u32>().ok()?;
            let b_syshide =
                if p[4].eq(HOTKEY_PARAM_SYSHIDE_T) { true }
                else if p[4].eq(HOTKEY_PARAM_SYSHIDE_F) { false }
                else { return None };
            let mut path = String::new();
            if p.len() > 5 { // pathの中にカンマを含んでいる場合の対処
                for i in 5..=p.len() - 1 { path += p[i]; }
            }
            Some(Self { hotkey_type: HotkeyType::IconLauncher, b_icon_large: b_large, w: w, h: h, b_sysfile_hidden: b_syshide, path: path })
        })() { return v };

        Self::default()
    }

    pub fn to_string(&self) -> String {
        let mut paramstr = String::default();
        if self.hotkey_type == HotkeyType::IconLauncher {
            paramstr += HOTKEY_PARAM_FILEICON;
            paramstr += ",";
            if self.b_icon_large {
                paramstr += HOTKEY_PARAM_ICON_LG;
            } else {
                paramstr += HOTKEY_PARAM_ICON_SM;
            }
            paramstr += ",";
        } else if self.hotkey_type == HotkeyType::ListLauncher {
            paramstr += HOTKEY_PARAM_FILELIST;
            paramstr += ",";
        } else if self.hotkey_type == HotkeyType::WinTaskList {
            paramstr += HOTKEY_PARAM_TASK;
            paramstr += ",";
        }

        paramstr += &(self.w.to_string() + ",");
        paramstr += &(self.h.to_string() + ",");

        if self.hotkey_type == HotkeyType::WinTaskList {
            return paramstr
        }

        if self.b_sysfile_hidden {
            paramstr += HOTKEY_PARAM_SYSHIDE_T;
        } else {
            paramstr += HOTKEY_PARAM_SYSHIDE_F;
        }

        if self.path.len() > 0 {
            paramstr += &(",".to_string() + &self.path);
        }
        paramstr
    }

    pub fn load_notify_icon_param() -> Option<(String, i32)> {
        let s = load_reg_sz(REGKEY_APP_PARAM, REG_NOTIFY_ICON);
        if s.len() == 0 {
            let _ = store_reg_sz(REGKEY_APP_PARAM, REG_NOTIFY_ICON, &s);
            return None
        }
        if s.split("|").count() < 2 {
            return Some((s, -1))
        } else {
            let part:Vec<_> = s.split("|").collect();
            let index = part[1].parse::<i32>();
            if index.is_ok() { return Some((part[0].to_string(), index.unwrap())) } else { Some((part[0].to_string(), -1)) }
        }
    }

    pub fn load_filesort_param(path: &str) -> Vec<String> {
        let mut v = Vec::<String>::default();
        load_reg_multi_sz(REGKEY_FILE_LIST_ORDER, path, &mut v);
        v
    }

    pub fn store_filesort_param(b_sorted: bool, path: &str, sort_list: &mut Vec<String>) -> Result<()> {
        let mut now = Self::load_filesort_param(path);
        let firstline = if now.len() > 0 {
            now.remove(0)
        } else {
            if !b_sorted { return Err(Error::OK) }
            Self::default().to_string()
        };

        let mut new = Vec::<String>::default();
        new.push(firstline);
        if b_sorted || now.len() > 1 {
            new.append(sort_list);
        }
        store_reg_multi_sz(REGKEY_FILE_LIST_ORDER, path, &new)
    }

    pub fn load_winsort_param(v: &mut Vec<WinSortList>) {
        let mut reg = Vec::<String>::default();
        load_reg_multi_sz(REGKEY_APP_PARAM, REG_WIN_SORT, &mut reg);
        for l in reg {
            if l.contains("*") {
                v.push(WinSortList::WILDCARD(l));
            } else {
                v.push(WinSortList::IMGFILE(l));
            }
        }
    }
    // s2 - wildcard
    pub fn compare_wildcard(s1: &str, s2: &str) -> bool {
        let mut idx = 0;
        let splen = s2.split("*").count(); // collect(アロケーション)はしない方が速い
        for (n, parts) in s2.split("*").enumerate() {
            if parts.len() == 0 {
                if n == splen - 1 { idx = s1.len(); } // 末尾の*は全マッチ
                continue
            }
            let before = idx;
            loop {
                match s1[idx..].find(parts) {
                    None => break,
                    Some(i) => idx += i + parts.len(),
                }
            }
            if idx == before { return false }
        }
        idx == s1.len()
    }

    pub fn contains_window_sort_list(vec: &Vec<WinSortList>, c: &str) -> Option<usize> {
        for (idx, v) in vec.iter().enumerate() {
            if match v {
                WinSortList::WILDCARD(i) => Self::compare_wildcard(c, i),
                WinSortList::IMGFILE(i) => i.eq(c),
            } { return Some(idx) }
        }
        None
    }

    pub fn contains_procimg_list(vec: &mut dyn Iterator<Item = &&String>, c: &str) -> Option<String> {
        for v in vec {
            if Self::compare_wildcard(c, v) { return Some(v.to_string()) }
        }
        None
    }

    pub fn store_winsort_param(v: &Vec<WinSortList>) {
        let sortlist = v.iter().map(|v| match v {
            WinSortList::WILDCARD(i) => i.clone(),
            WinSortList::IMGFILE(i) => i.clone(),
        }).collect::<Vec<_>>();
        let _ = store_reg_multi_sz(REGKEY_APP_PARAM, REG_WIN_SORT, &sortlist);
    }

    pub fn update_dir_param(path: &str, param:String) -> Result<()> {
        let mut now = Self::load_filesort_param(path);
        let now_len = now.len();
        if  now_len > 0 {
            now.remove(0);
        }

        let mut new = Vec::<String>::default();
        new.push(param);
        if now_len > 1 {
            for i in now {
                new.push(i);
            }
        }
        store_reg_multi_sz(REGKEY_FILE_LIST_ORDER, path, &new)
    }

    pub fn sort_reset(path: &str) -> Result<()> {
        let mut now = Self::load_filesort_param(path);
        if now.len() <= 1 { return Err(Error::OK) }

        let firstline = now.remove(0);
        let mut val = Vec::<String>::default();
        val.push(firstline);
        store_reg_multi_sz(REGKEY_FILE_LIST_ORDER, path, &val)
    }

    pub fn check_hotkey_char(c: &char) -> bool {
        for vc in VK_CHARS {
            if vc.eq(c) { return true }
        }
        false
    }

    pub fn conv_char2vmod(c: char) -> Result<HOT_KEY_MODIFIERS> {
        if c.eq(&HKMOD_CHAR_SHIFT) {
            Ok(MOD_SHIFT | MOD_CONTROL)
        } else if c.eq(&HKMOD_CHAR_ALT) {
            Ok(MOD_ALT | MOD_CONTROL)
        } else if c.eq(&HKMOD_CHAR_NONE) {
            Ok(HOT_KEY_MODIFIERS(0))
        } else {
            Err(Error::OK)
        }
    }

    pub fn conv_vmod2char(hkmod: HOT_KEY_MODIFIERS) -> Result<char> {
        if hkmod.eq(&(MOD_SHIFT | MOD_CONTROL)) {
            Ok(HKMOD_CHAR_SHIFT)
        } else if hkmod.eq(&(MOD_ALT | MOD_CONTROL)) {
            Ok(HKMOD_CHAR_ALT)
        } else if hkmod.0 == 0 {
            Ok(HKMOD_CHAR_NONE)
        } else {
            Err(Error::OK)
        }
    }

    pub fn conv_char2vkey(c: char) -> Result<VIRTUAL_KEY> {
        if c.is_ascii_uppercase() {
            let mut c_buf = [0u8];
            c.encode_utf8(&mut c_buf);
            let a_buf: Vec<u8> = "A".bytes().collect();
            Ok(VIRTUAL_KEY(VK_A.0 as u16 + (c_buf[0] - a_buf[0]) as u16))
        } else if c.is_numeric() {
            let mut c_buf = [0u8];
            c.encode_utf8(&mut c_buf);
            let a_buf: Vec<u8> = "0".bytes().collect();
            Ok(VIRTUAL_KEY(VK_0.0 as u16 + (c_buf[0] - a_buf[0]) as u16))
        } else {
            for (idx, vc) in VK_CHARS.iter().enumerate() {
                if c.eq(&vc) { return Ok(VK_LIST[idx]) }
            }
            Err(Error::OK)
        }
    }

    pub fn conv_vkey2char(vkey: VIRTUAL_KEY) -> Result<char> {
        let mut c_buf: Vec<u16> = "A".encode_utf16().collect();
        if VK_A.0 <= vkey.0 && vkey.0 <= VK_Z.0 {
            c_buf[0] += vkey.0 - VK_A.0;
            Ok(String::from_utf16(&c_buf).unwrap().chars().nth(0).unwrap())
        } else if VK_0.0 <= vkey.0 && vkey.0 <= VK_9.0 {
            c_buf = "0".encode_utf16().collect();
            c_buf[0] += vkey.0 - VK_0.0;
            Ok(String::from_utf16(&c_buf).unwrap().chars().nth(0).unwrap())
        } else {
            for (idx, vk) in VK_LIST.iter().enumerate() {
                if vkey.eq(&vk) { return Ok(VK_CHARS[idx]) }
            }
            Err(Error::OK)
        }
    }

    pub fn enum_hotkey_param() -> Vec<(HOT_KEY_MODIFIERS, VIRTUAL_KEY, String/*viewParam*/)> {
        let mut ret = Vec::<(HOT_KEY_MODIFIERS, VIRTUAL_KEY /*VKEY*/, String/*viewParam*/)>::default();

        let mut h = RegKey(HKEY(0)); // auto drop resouce
        let r = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, WSTR::from(REGKEY_APP_PARAM).PCWSTR(), 0, KEY_READ, &mut h.0) };
        if r.is_err() { return ret }

        let mut idx: u32 = 0;
        let mut valname = [0u16; 128];
        let mut valdata = [0u16; 64 * 1024];
        loop {
            let mut typeval = 0u32;
            let mut size_valname = valname.len() as u32;
            let mut size_valdata = valdata.len() as u32;

            let result = unsafe { RegEnumValueW(h.0, idx, PWSTR::from_raw(&mut valname as *mut _ as _), &mut size_valname,
                None, Some(&mut typeval), Some(&mut valdata as *mut _ as _), Some(&mut size_valdata)) };

            if let Err(ecode) = result {
                if ecode == ERROR_NO_MORE_ITEMS.into() { break; }
                if ecode == ERROR_MORE_DATA.into() { idx += 1; continue; }
            }
            if typeval != REG_SZ.0 {
                idx += 1; continue;
            }

            let p = WSTR::from_slice_to_string(&valname, size_valname as usize);
            let v = WSTR::from_slice_to_string(&valdata, ((size_valdata - 2) / 2) as usize);
            let vk_mod = Self::conv_char2vmod(p.chars().nth(0).unwrap());
            let vk_code = Self::conv_char2vkey(p.chars().nth(1).unwrap());
            if vk_mod.is_ok() && vk_code.is_ok() {
                ret.push((vk_mod.unwrap(),vk_code.unwrap(), v));
            }
            idx += 1;
        }
        ret
    }

    pub fn store_hotkey_param(v: Vec::<(HOT_KEY_MODIFIERS, VIRTUAL_KEY, PropertyHolder)>) -> Result<()> {
        let mut del_valnames = Vec::<String>::default();

        let params = Self::enum_hotkey_param();
        for v in params {
            let m = Self::conv_vmod2char(v.0).unwrap().to_string();
            let k = Self::conv_vkey2char(v.1).unwrap().to_string();
            del_valnames.push(m + &k);
        }

        for valname in &del_valnames {
            let _ = delete_reg(REGKEY_APP_PARAM, valname);
        }

        for p in v {
            let m = Self::conv_vmod2char(p.0).unwrap().to_string();
            let k = Self::conv_vkey2char(p.1).unwrap().to_string();
            let _ = store_reg_sz(REGKEY_APP_PARAM, &(m + &k), &p.2.to_string());
        }
        Ok(())
    }
}
