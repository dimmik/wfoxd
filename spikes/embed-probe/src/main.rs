//! wfoxd probe 3 — одноразовый спайк overlay-встраивания под Windows 11.
//!
//! Сайдбар с кнопками запуска, полоса табов, каждое запущенное приложение
//! встраивается overlay'ем (owned-окно без рамки поверх рабочей области).
//! Окно ищется: 1) среди процессов Job Object, 2) как новое видимое окно,
//! которого не было до запуска (Store-приложения, Explorer).
#![windows_subsystem = "windows"]
#![allow(non_snake_case, clippy::upper_case_acronyms)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};

type HWND = *mut c_void;
type HANDLE = *mut c_void;
type BOOL = i32;

#[repr(C)]
struct WNDCLASSW {
    style: u32,
    wndproc: extern "system" fn(HWND, u32, usize, isize) -> isize,
    cls_extra: i32,
    wnd_extra: i32,
    inst: HANDLE,
    icon: HANDLE,
    cursor: HANDLE,
    bg: HANDLE,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
struct MSG {
    hwnd: HWND,
    msg: u32,
    w: usize,
    l: isize,
    time: u32,
    pt: POINT,
    private: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct POINT {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
struct RECT {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}
#[repr(C)]
struct PAINTSTRUCT {
    hdc: HANDLE,
    erase: BOOL,
    rc: RECT,
    restore: BOOL,
    inc: BOOL,
    reserved: [u8; 32],
}
#[repr(C)]
struct STARTUPINFOW {
    cb: u32,
    reserved: *mut u16,
    desktop: *mut u16,
    title: *mut u16,
    x: u32,
    y: u32,
    xs: u32,
    ys: u32,
    xc: u32,
    yc: u32,
    fill: u32,
    flags: u32,
    show: u16,
    cb2: u16,
    reserved2: *mut u8,
    stdin: HANDLE,
    stdout: HANDLE,
    stderr: HANDLE,
}
#[repr(C)]
struct PROCESS_INFORMATION {
    process: HANDLE,
    thread: HANDLE,
    pid: u32,
    tid: u32,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(c: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(ex: u32, cls: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: HANDLE, inst: HANDLE, param: *mut c_void) -> HWND;
    fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
    fn DestroyWindow(h: HWND) -> BOOL;
    fn ShowWindow(h: HWND, cmd: i32) -> BOOL;
    fn GetMessageW(m: *mut MSG, h: HWND, a: u32, b: u32) -> BOOL;
    fn TranslateMessage(m: *const MSG) -> BOOL;
    fn DispatchMessageW(m: *const MSG) -> isize;
    fn PostQuitMessage(code: i32);
    fn PostMessageW(h: HWND, m: u32, w: usize, l: isize) -> BOOL;
    fn BeginPaint(h: HWND, ps: *mut PAINTSTRUCT) -> HANDLE;
    fn EndPaint(h: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    fn FillRect(dc: HANDLE, r: *const RECT, brush: HANDLE) -> i32;
    fn DrawTextW(dc: HANDLE, s: *const u16, n: i32, r: *mut RECT, flags: u32) -> i32;
    fn InvalidateRect(h: HWND, r: *const RECT, erase: BOOL) -> BOOL;
    fn GetClientRect(h: HWND, r: *mut RECT) -> BOOL;
    fn GetWindowRect(h: HWND, r: *mut RECT) -> BOOL;
    fn ClientToScreen(h: HWND, p: *mut POINT) -> BOOL;
    fn SetTimer(h: HWND, id: usize, ms: u32, f: *const c_void) -> usize;
    fn EnumWindows(f: extern "system" fn(HWND, isize) -> BOOL, l: isize) -> BOOL;
    fn GetWindowThreadProcessId(h: HWND, pid: *mut u32) -> u32;
    fn IsWindow(h: HWND) -> BOOL;
    fn IsWindowVisible(h: HWND) -> BOOL;
    fn IsIconic(h: HWND) -> BOOL;
    fn IsZoomed(h: HWND) -> BOOL;
    fn GetWindow(h: HWND, cmd: u32) -> HWND;
    fn GetWindowLongPtrW(h: HWND, i: i32) -> isize;
    fn SetWindowLongPtrW(h: HWND, i: i32, v: isize) -> isize;
    fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, w: i32, h2: i32, flags: u32) -> BOOL;
    fn ShowWindowAsync(h: HWND, cmd: i32) -> BOOL;
    fn SetForegroundWindow(h: HWND) -> BOOL;
    fn GetWindowTextW(h: HWND, buf: *mut u16, n: i32) -> i32;
    fn LoadCursorW(inst: HANDLE, name: *const u16) -> HANDLE;
    fn SetProcessDpiAwarenessContext(ctx: HANDLE) -> BOOL;
    fn GetDpiForWindow(h: HWND) -> u32;
    fn GetClassNameW(h: HWND, buf: *mut u16, n: i32) -> i32;
    fn RedrawWindow(h: HWND, r: *const RECT, rgn: HANDLE, flags: u32) -> BOOL;
}
#[link(name = "dwmapi")]
extern "system" {
    fn DwmGetWindowAttribute(h: HWND, attr: u32, out: *mut c_void, cb: u32) -> i32;
}
#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(c: u32) -> HANDLE;
    fn DeleteObject(o: HANDLE) -> BOOL;
    fn SelectObject(dc: HANDLE, o: HANDLE) -> HANDLE;
    fn CreateCompatibleDC(dc: HANDLE) -> HANDLE;
    fn CreateCompatibleBitmap(dc: HANDLE, w: i32, h: i32) -> HANDLE;
    fn DeleteDC(dc: HANDLE) -> BOOL;
    fn BitBlt(dc: HANDLE, x: i32, y: i32, w: i32, h: i32, src: HANDLE, sx: i32, sy: i32, rop: u32) -> BOOL;
    fn SetBkMode(dc: HANDLE, mode: i32) -> i32;
    fn SetTextColor(dc: HANDLE, c: u32) -> u32;
    fn CreateFontW(h: i32, w: i32, esc: i32, orient: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, outp: u32, clipp: u32, quality: u32, pitch: u32, face: *const u16) -> HANDLE;
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> HANDLE;
    fn CreateProcessW(app: *const u16, cmd: *mut u16, pa: *mut c_void, ta: *mut c_void, inherit: BOOL, flags: u32, env: *mut c_void, cwd: *const u16, si: *mut STARTUPINFOW, pi: *mut PROCESS_INFORMATION) -> BOOL;
    fn ResumeThread(h: HANDLE) -> u32;
    fn CloseHandle(h: HANDLE) -> BOOL;
    fn CreateJobObjectW(attrs: *mut c_void, name: *const u16) -> HANDLE;
    fn AssignProcessToJobObject(job: HANDLE, process: HANDLE) -> BOOL;
    fn QueryInformationJobObject(job: HANDLE, class: u32, info: *mut c_void, len: u32, ret: *mut u32) -> BOOL;
    fn GetTickCount() -> u32;
    fn GetLastError() -> u32;
    fn GetCurrentProcess() -> HANDLE;
    fn TerminateProcess(h: HANDLE, code: u32) -> BOOL;
    fn OpenProcess(access: u32, inherit: BOOL, pid: u32) -> HANDLE;
    fn QueryFullProcessImageNameW(h: HANDLE, flags: u32, buf: *mut u16, len: *mut u32) -> BOOL;
}

const WS_CAPTION: isize = 0x00C0_0000;
const WS_THICKFRAME: isize = 0x0004_0000;
const WS_VISIBLE: isize = 0x1000_0000;
const WS_EX_TOOLWINDOW: isize = 0x80;
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;
const GWLP_HWNDPARENT: i32 = -8;
const SWP_NOSIZE: u32 = 0x1;
const SWP_NOMOVE: u32 = 0x2;
const SWP_NOZORDER: u32 = 0x4;
const SWP_NOACTIVATE: u32 = 0x10;
const SWP_FRAMECHANGED: u32 = 0x20;
const SWP_ASYNC: u32 = 0x4000;
const DT_LEFT_LINE: u32 = 0x20 | 0x4 | 0x8000 | 0x800; // SINGLELINE|VCENTER|END_ELLIPSIS|NOPREFIX
const DT_CENTER_LINE: u32 = DT_LEFT_LINE | 0x1;

/// (подпись, командная строка). cmd идёт через conhost, иначе Windows 11 может отдать его в Windows Terminal.
const PROFILES: &[(&str, &str)] = &[
    ("cmd (conhost)", "conhost.exe cmd.exe /k echo wfoxd: WFOXD_SESSION=%WFOXD_SESSION%"),
    ("cmd (как есть)", "cmd.exe /k echo wfoxd: WFOXD_SESSION=%WFOXD_SESSION%"),
    ("Блокнот", "notepad.exe"),
    ("Paint", "mspaint.exe"),
    ("Проводник", "explorer.exe"),
    ("Таблица символов", "charmap.exe"),
    ("PowerShell (conhost)", "conhost.exe powershell.exe -NoExit -Command \"$env:WFOXD_SESSION\""),
];
const SESSION_VAR: (&str, &str) = ("WFOXD_SESSION", "probe");
const FIND_TIMEOUT_MS: u32 = 15_000;
/// Сколько ещё следим за запуском после встраивания первого окна (вдруг приложение откроет второе).
const GRACE_MS: u32 = 3_000;

struct Tab {
    hwnd: HWND,
    title: String,
    style: isize,
    exstyle: isize,
    stripped: bool,
}
struct Pending {
    label: &'static str,
    job: HANDLE,
    before: Vec<HWND>,
    seen: Vec<HWND>,
    t0: u32,
    attached: Option<u32>,
}
struct State {
    main: HWND,
    scale: f32,
    font: HANDLE,
    tabs: Vec<Tab>,
    active: usize,
    pending: Vec<Pending>,
    status: String,
    cwd: String,
    strip_frame: bool,
    log_path: String,
    t_start: u32,
}
enum Hit {
    Launch(usize),
    Tab(usize),
    Close(usize),
    Crash,
    ToggleFrame,
    Nothing,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn inside(r: &RECT, x: i32, y: i32) -> bool {
    x >= r.l && x < r.r && y >= r.t && y < r.b
}

impl State {
    fn s(&self, v: i32) -> i32 {
        (v as f32 * self.scale).round() as i32
    }
    fn client(&self) -> RECT {
        let mut r = RECT::default();
        unsafe { GetClientRect(self.main, &mut r) };
        r
    }
    fn sidebar_w(&self) -> i32 {
        self.s(210)
    }
    fn tabs_h(&self) -> i32 {
        self.s(38)
    }
    fn session_rect(&self) -> RECT {
        RECT { l: self.s(10), t: self.s(10), r: self.sidebar_w() - self.s(10), b: self.s(100) }
    }
    fn launch_rect(&self, i: usize) -> RECT {
        let y = self.s(136) + i as i32 * self.s(36);
        RECT { l: self.s(10), t: y, r: self.sidebar_w() - self.s(10), b: y + self.s(30) }
    }
    fn toggle_rect(&self) -> RECT {
        let r = self.launch_rect(PROFILES.len());
        RECT { t: r.t + self.s(10), b: r.b + self.s(10), ..r }
    }
    fn crash_rect(&self) -> RECT {
        let c = self.client();
        RECT { l: self.s(10), t: c.b - self.s(42), r: self.sidebar_w() - self.s(10), b: c.b - self.s(12) }
    }
    fn tab_rect(&self, i: usize) -> RECT {
        let w = self.s(180);
        let x = self.sidebar_w() + self.s(4) + i as i32 * (w + self.s(4));
        RECT { l: x, t: self.s(5), r: x + w, b: self.tabs_h() }
    }
    fn close_rect(&self, i: usize) -> RECT {
        let t = self.tab_rect(i);
        RECT { l: t.r - self.s(28), ..t }
    }
    fn work_rect(&self) -> RECT {
        let c = self.client();
        RECT { l: self.sidebar_w(), t: self.tabs_h(), r: c.r, b: c.b }
    }
    fn hit(&self, x: i32, y: i32) -> Hit {
        if let Some(i) = (0..PROFILES.len()).find(|&i| inside(&self.launch_rect(i), x, y)) {
            return Hit::Launch(i);
        }
        if inside(&self.crash_rect(), x, y) {
            return Hit::Crash;
        }
        if inside(&self.toggle_rect(), x, y) {
            return Hit::ToggleFrame;
        }
        for i in 0..self.tabs.len() {
            if inside(&self.close_rect(i), x, y) {
                return Hit::Close(i);
            }
            if inside(&self.tab_rect(i), x, y) {
                return Hit::Tab(i);
            }
        }
        Hit::Nothing
    }
}

unsafe fn fill(dc: HANDLE, r: RECT, color: u32) {
    let b = CreateSolidBrush(color);
    FillRect(dc, &r, b);
    DeleteObject(b);
}
unsafe fn text(dc: HANDLE, mut r: RECT, s: &str, color: u32, flags: u32) {
    SetTextColor(dc, color);
    let v: Vec<u16> = s.encode_utf16().collect();
    DrawTextW(dc, v.as_ptr(), v.len() as i32, &mut r, flags);
}
unsafe fn make_font(scale: f32) -> HANDLE {
    let face = w("Segoe UI");
    CreateFontW(-(15.0 * scale) as i32, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr())
}

unsafe fn paint(st: &State) {
    let mut ps: PAINTSTRUCT = zeroed();
    let dc = BeginPaint(st.main, &mut ps);
    let c = st.client();
    let (wd, ht) = (c.r.max(1), c.b.max(1));
    let mem = CreateCompatibleDC(dc);
    let bmp = CreateCompatibleBitmap(dc, wd, ht);
    let old_bmp = SelectObject(mem, bmp);
    let old_font = SelectObject(mem, st.font);
    SetBkMode(mem, 1);

    const WHITE: u32 = 0x00F0_F0F0;
    const DIM: u32 = 0x00B0_A8A0;
    fill(mem, c, 0x0026_2220);
    fill(mem, RECT { l: 0, t: 0, r: st.sidebar_w(), b: c.b }, 0x003A_2A1E);
    fill(mem, RECT { l: st.sidebar_w(), t: 0, r: c.r, b: st.tabs_h() }, 0x0030_2A26);

    // плашка сессии
    let sr = st.session_rect();
    fill(mem, sr, 0x0080_5A34);
    let line = |i: i32| RECT { l: sr.l + st.s(10), t: sr.t + st.s(6) + i * st.s(26), r: sr.r - st.s(6), b: sr.t + st.s(6) + (i + 1) * st.s(26) };
    text(mem, line(0), "Сессия «probe»", WHITE, DT_LEFT_LINE);
    text(mem, line(1), &format!("{}={}", SESSION_VAR.0, SESSION_VAR.1), DIM, DT_LEFT_LINE);
    text(mem, line(2), &st.cwd, DIM, DT_LEFT_LINE);

    text(mem, RECT { l: st.s(12), t: st.s(106), r: st.sidebar_w(), b: st.s(132) }, "Запустить в сессии:", DIM, DT_LEFT_LINE);
    for (i, (label, _)) in PROFILES.iter().enumerate() {
        let r = st.launch_rect(i);
        fill(mem, r, 0x0055_4030);
        text(mem, RECT { l: r.l + st.s(10), ..r }, &format!("+  {label}"), WHITE, DT_LEFT_LINE);
    }
    let tr = st.toggle_rect();
    fill(mem, tr, 0x0050_5A30);
    text(mem, RECT { l: tr.l + st.s(10), ..tr }, &format!("Рамку окна: {}", if st.strip_frame { "снимать" } else { "оставлять" }), WHITE, DT_LEFT_LINE);
    let cr = st.crash_rect();
    text(mem, RECT { l: st.s(12), t: cr.t - st.s(54), r: st.sidebar_w() - st.s(6), b: cr.t - st.s(30) }, "ПКМ по табу — отцепить", DIM, DT_LEFT_LINE);
    text(mem, RECT { l: st.s(12), t: cr.t - st.s(30), r: st.sidebar_w() - st.s(6), b: cr.t - st.s(6) }, &format!("Лог: {}", st.log_path), DIM, DT_LEFT_LINE);
    fill(mem, cr, 0x0030_30A8);
    text(mem, cr, "Убить wfoxd (тест сбоя)", WHITE, DT_CENTER_LINE);

    for (i, t) in st.tabs.iter().enumerate() {
        let r = st.tab_rect(i);
        fill(mem, r, if i == st.active { 0x0026_2220 } else { 0x0045_3A32 });
        let title = if t.title.is_empty() { "(без заголовка)" } else { &t.title };
        text(mem, RECT { l: r.l + st.s(10), r: r.r - st.s(28), ..r }, title, WHITE, DT_LEFT_LINE);
        text(mem, st.close_rect(i), "×", DIM, DT_CENTER_LINE);
    }

    let msg = if let Some(p) = st.pending.iter().find(|p| p.attached.is_none()) {
        format!("Жду окно: {}…", p.label)
    } else if !st.status.is_empty() {
        st.status.clone()
    } else if st.tabs.is_empty() {
        "Табов нет. Слева выбери, что запустить.".to_owned()
    } else {
        String::new()
    };
    if !msg.is_empty() {
        if st.tabs.is_empty() {
            text(mem, st.work_rect(), &msg, DIM, DT_CENTER_LINE);
        } else {
            // рабочая область закрыта встроенным окном — статус пишем в полосе табов справа
            let l = st.tab_rect(st.tabs.len() - 1).r + st.s(12);
            text(mem, RECT { l, t: 0, r: c.r - st.s(12), b: st.tabs_h() }, &msg, 0x0060_C0F0, DT_LEFT_LINE);
        }
    }

    BitBlt(dc, 0, 0, wd, ht, mem, 0, 0, 0x00CC_0020);
    SelectObject(mem, old_font);
    SelectObject(mem, old_bmp);
    DeleteObject(bmp);
    DeleteDC(mem);
    EndPaint(st.main, &ps);
}

extern "system" fn collect(h: HWND, l: isize) -> BOOL {
    unsafe { (*(l as *mut Vec<HWND>)).push(h) };
    1
}
fn top_windows() -> Vec<HWND> {
    let mut v: Vec<HWND> = Vec::new();
    unsafe { EnumWindows(collect, &mut v as *mut Vec<HWND> as isize) };
    v
}
fn pid_of(h: HWND) -> u32 {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(h, &mut pid) };
    pid
}
fn job_pids(job: HANDLE) -> Vec<u32> {
    // JOBOBJECT_BASIC_PROCESS_ID_LIST: DWORD assigned, DWORD in_list, ULONG_PTR ids[]
    let mut buf = [0u64; 129];
    let ok = unsafe { QueryInformationJobObject(job, 3, buf.as_mut_ptr().cast(), size_of::<[u64; 129]>() as u32, null_mut()) };
    if ok == 0 {
        return Vec::new();
    }
    let n = ((buf[0] >> 32) as usize).min(128);
    buf[1..=n].iter().map(|&p| p as u32).collect()
}
fn window_text(h: HWND) -> String {
    let mut buf = [0u16; 256];
    // Для окна чужого процесса GetWindowTextW не шлёт WM_GETTEXT — зависшее приложение нас не подвесит.
    let n = unsafe { GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}
fn process_name(pid: u32) -> String {
    unsafe {
        let h = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
        if h.is_null() {
            return "?".into();
        }
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(h);
        if ok == 0 {
            return "?".into();
        }
        let full = String::from_utf16_lossy(&buf[..len as usize]);
        full.rsplit('\\').next().unwrap_or(&full).to_owned()
    }
}
fn describe(h: HWND) -> String {
    unsafe {
        let mut cls = [0u16; 128];
        let n = GetClassNameW(h, cls.as_mut_ptr(), cls.len() as i32);
        let mut r = RECT::default();
        GetWindowRect(h, &mut r);
        let pid = pid_of(h);
        format!(
            "hwnd={:#x} pid={} ({}) class=\"{}\" title=\"{}\" style={:#010x} ex={:#010x} owner={:#x} rect=({},{} {}x{})",
            h as usize, pid, process_name(pid), String::from_utf16_lossy(&cls[..n.max(0) as usize]), window_text(h),
            GetWindowLongPtrW(h, GWL_STYLE) as u32, GetWindowLongPtrW(h, GWL_EXSTYLE) as u32, GetWindow(h, 4) as usize,
            r.l, r.t, r.r - r.l, r.b - r.t
        )
    }
}
fn log(st: &State, line: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&st.log_path) {
        let t = unsafe { GetTickCount() }.wrapping_sub(st.t_start);
        let _ = writeln!(f, "{:>5}.{:03} {line}\r", t / 1000, t % 1000);
    }
}
fn pick_log_path() -> String {
    let dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    for d in dir.into_iter().chain(std::env::var_os("TEMP").map(Into::into)) {
        let p = d.join("wfoxd-probe.log");
        if std::fs::OpenOptions::new().create(true).append(true).open(&p).is_ok() {
            return p.display().to_string();
        }
    }
    "wfoxd-probe.log".into()
}

unsafe fn embeddable(st: &State, h: HWND) -> bool {
    if h == st.main || IsWindowVisible(h) == 0 || !GetWindow(h, 4).is_null() || st.tabs.iter().any(|t| t.hwnd == h) {
        return false;
    }
    let style = GetWindowLongPtrW(h, GWL_STYLE);
    let ex = GetWindowLongPtrW(h, GWL_EXSTYLE);
    style & WS_CAPTION == WS_CAPTION && ex & WS_EX_TOOLWINDOW == 0
}

fn env_block() -> Vec<u16> {
    let mut vars: Vec<(Vec<u16>, Vec<u16>)> = std::env::vars_os()
        .filter(|(k, _)| !k.to_string_lossy().eq_ignore_ascii_case(SESSION_VAR.0))
        .map(|(k, v)| (k.encode_wide().collect(), v.encode_wide().collect()))
        .collect();
    vars.push((SESSION_VAR.0.encode_utf16().collect(), SESSION_VAR.1.encode_utf16().collect()));
    vars.sort_by_key(|(k, _)| String::from_utf16_lossy(k).to_uppercase());
    let mut out = Vec::new();
    for (k, v) in vars {
        out.extend(k);
        out.push(u16::from(b'='));
        out.extend(v);
        out.push(0);
    }
    out.push(0);
    out
}

unsafe fn launch(st: &mut State, i: usize) {
    let (label, cmd) = PROFILES[i];
    let before: Vec<HWND> = top_windows().into_iter().filter(|&h| IsWindowVisible(h) != 0).collect();
    let job = CreateJobObjectW(null_mut(), null());
    let mut env = env_block();
    let mut cmdline = w(cmd);
    let cwd = w(&st.cwd);
    let mut si: STARTUPINFOW = zeroed();
    si.cb = size_of::<STARTUPINFOW>() as u32;
    let mut pi: PROCESS_INFORMATION = zeroed();
    const CREATE_SUSPENDED: u32 = 0x4;
    const CREATE_UNICODE_ENVIRONMENT: u32 = 0x400;
    log(st, &format!("ЗАПУСК [{label}] {cmd}"));
    let ok = CreateProcessW(null(), cmdline.as_mut_ptr(), null_mut(), null_mut(), 0, CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT, env.as_mut_ptr().cast(), cwd.as_ptr(), &mut si, &mut pi);
    if ok == 0 {
        st.status = format!("{label}: не запустилось (ошибка {})", GetLastError());
        log(st, &format!("  ошибка: {}", st.status));
        CloseHandle(job);
        return;
    }
    let in_job = AssignProcessToJobObject(job, pi.process) != 0;
    log(st, &format!("  pid={} в job: {}", pi.pid, in_job));
    ResumeThread(pi.thread);
    CloseHandle(pi.thread);
    CloseHandle(pi.process);
    st.status.clear();
    st.pending.push(Pending { label, job, before, seen: Vec::new(), t0: GetTickCount(), attached: None });
}

unsafe fn attach(st: &mut State, h: HWND, why: &str) {
    let style = GetWindowLongPtrW(h, GWL_STYLE);
    let exstyle = GetWindowLongPtrW(h, GWL_EXSTYLE);
    let stripped = st.strip_frame;
    log(st, &format!("  ВСТРАИВАЮ ({why}, рамку {}): {}", if stripped { "снимаю" } else { "оставляю" }, describe(h)));
    if IsZoomed(h) != 0 || IsIconic(h) != 0 {
        ShowWindowAsync(h, 9); // SW_RESTORE
    }
    SetWindowLongPtrW(h, GWLP_HWNDPARENT, st.main as isize);
    if stripped {
        SetWindowLongPtrW(h, GWL_STYLE, style & !(WS_CAPTION | WS_THICKFRAME));
        SetWindowPos(h, null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_ASYNC);
    }
    st.tabs.push(Tab { hwnd: h, title: window_text(h), style, exstyle, stripped });
    st.active = st.tabs.len() - 1;
    layout(st);
    SetForegroundWindow(h);
}

/// Вернуть окну рамку и отпустить его. `offset` — сдвиг, чтобы отцепленное окно было видно.
unsafe fn detach(st: &State, t: &Tab, offset: i32) {
    let h = t.hwnd;
    if IsWindow(h) == 0 {
        return;
    }
    SetWindowLongPtrW(h, GWLP_HWNDPARENT, 0);
    if t.stripped {
        let vis = GetWindowLongPtrW(h, GWL_STYLE) & WS_VISIBLE;
        SetWindowLongPtrW(h, GWL_STYLE, (t.style & !WS_VISIBLE) | vis);
        SetWindowLongPtrW(h, GWL_EXSTYLE, t.exstyle);
    }
    SetWindowPos(h, null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_ASYNC);
    // «пинок»: изменение размера заставляет приложение пересчитать non-client область
    let mut r = RECT::default();
    GetWindowRect(h, &mut r);
    let (x, y, w, ht) = (r.l + offset, r.t + offset, r.r - r.l, r.b - r.t);
    SetWindowPos(h, null_mut(), x, y, w - 1, ht - 1, SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNC);
    SetWindowPos(h, null_mut(), x, y, w, ht, SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNC);
    RedrawWindow(h, null(), null_mut(), 0x1 | 0x400 | 0x80); // INVALIDATE|FRAME|ALLCHILDREN
    ShowWindowAsync(h, 5); // SW_SHOW
    log(st, &format!("ОТЦЕПИЛ (рамка была {}): {}", if t.stripped { "снята" } else { "своя" }, describe(h)));
}

unsafe fn detach_all(st: &mut State) {
    let tabs = std::mem::take(&mut st.tabs);
    for (i, t) in tabs.iter().enumerate() {
        detach(st, t, st.s(30) * i as i32);
    }
}

/// Держим активный таб над рабочей областью, остальные прячем. Зовётся и по таймеру — это же
/// обратная коррекция, если приложение само себя сдвинуло.
unsafe fn layout(st: &State) {
    if IsIconic(st.main) != 0 {
        return;
    }
    let wr = st.work_rect();
    let mut tl = POINT { x: wr.l, y: wr.t };
    ClientToScreen(st.main, &mut tl);
    let target = RECT { l: tl.x, t: tl.y, r: tl.x + (wr.r - wr.l), b: tl.y + (wr.b - wr.t) };
    for (i, t) in st.tabs.iter().enumerate() {
        let visible = IsWindowVisible(t.hwnd) != 0;
        if i == st.active {
            let mut cur = RECT::default();
            GetWindowRect(t.hwnd, &mut cur);
            // У окон Win10/11 вокруг видимой рамки есть невидимые поля DWM — учитываем их,
            // чтобы видимая часть окна совпала с рабочей областью.
            let mut ext = cur;
            if DwmGetWindowAttribute(t.hwnd, 9, (&mut ext as *mut RECT).cast(), size_of::<RECT>() as u32) != 0 || !visible {
                ext = cur;
            }
            let want = RECT { l: target.l - (ext.l - cur.l), t: target.t - (ext.t - cur.t), r: target.r + (cur.r - ext.r), b: target.b + (cur.b - ext.b) };
            if cur != want {
                SetWindowPos(t.hwnd, null_mut(), want.l, want.t, want.r - want.l, want.b - want.t, SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNC);
            }
            if !visible {
                ShowWindowAsync(t.hwnd, 8); // SW_SHOWNA
            }
        } else if visible {
            ShowWindowAsync(t.hwnd, 0); // SW_HIDE
        }
    }
}

unsafe fn tick(st: &mut State) {
    let mut dirty = false;
    let before = st.tabs.len();
    let dead: Vec<String> = st.tabs.iter().filter(|t| IsWindow(t.hwnd) == 0).map(|t| format!("ТАБ ЗАКРЫТ: hwnd={:#x} \"{}\"", t.hwnd as usize, t.title)).collect();
    for d in &dead {
        log(st, d);
    }
    st.tabs.retain(|t| IsWindow(t.hwnd) != 0);
    if st.tabs.len() != before {
        dirty = true;
        st.active = st.active.min(st.tabs.len().saturating_sub(1));
    }
    for t in &mut st.tabs {
        let title = window_text(t.hwnd);
        if title != t.title {
            t.title = title;
            dirty = true;
        }
    }
    if !st.pending.is_empty() {
        let wins = top_windows();
        let now = GetTickCount();
        let mut i = 0;
        while i < st.pending.len() {
            // логируем каждое новое видимое окно верхнего уровня, появившееся после запуска
            let fresh: Vec<HWND> = wins.iter().copied().filter(|&h| h != st.main && IsWindowVisible(h) != 0 && !st.pending[i].before.contains(&h) && !st.pending[i].seen.contains(&h)).collect();
            for h in fresh {
                st.pending[i].seen.push(h);
                let line = format!("  [{}] новое окно: {}", st.pending[i].label, describe(h));
                log(st, &line);
            }
            let found = {
                let p = &st.pending[i];
                let pids = job_pids(p.job);
                let cands: Vec<HWND> = wins.iter().copied().filter(|&h| embeddable(st, h)).collect();
                cands.iter().copied().find(|&h| pids.contains(&pid_of(h))).or_else(|| cands.iter().copied().find(|h| !p.before.contains(h)))
            };
            let p = &st.pending[i];
            let attached = p.attached;
            let age = now.wrapping_sub(attached.unwrap_or(p.t0));
            match (found, attached) {
                (Some(h), None) => {
                    st.pending[i].attached = Some(now);
                    attach(st, h, "основное");
                    dirty = true;
                    i += 1;
                }
                (Some(h), Some(_)) => {
                    attach(st, h, "дополнительное, тот же запуск");
                    dirty = true;
                    i += 1;
                }
                (None, None) if age > FIND_TIMEOUT_MS => {
                    let p = st.pending.remove(i);
                    CloseHandle(p.job);
                    st.status = format!("{}: окно не нашлось за {} с", p.label, FIND_TIMEOUT_MS / 1000);
                    log(st, &format!("  [{}] ТАЙМАУТ, окно не нашлось", p.label));
                    dirty = true;
                }
                (None, Some(_)) if age > GRACE_MS => {
                    let p = st.pending.remove(i);
                    CloseHandle(p.job);
                    log(st, &format!("  [{}] больше не слежу", p.label));
                }
                _ => i += 1,
            }
        }
    }
    layout(st);
    if dirty {
        InvalidateRect(st.main, null(), 0);
    }
}

unsafe fn handle(st: &mut State, m: u32, wp: usize, lp: isize) -> Option<isize> {
    match m {
        0x000F => {
            paint(st);
            Some(0)
        }
        0x0014 => Some(1), // WM_ERASEBKGND
        0x0113 => {
            tick(st);
            Some(0)
        }
        0x0003 | 0x0005 => {
            // WM_MOVE / WM_SIZE
            layout(st);
            InvalidateRect(st.main, null(), 0);
            None
        }
        0x0201 => {
            let x = (lp & 0xFFFF) as i16 as i32;
            let y = ((lp >> 16) & 0xFFFF) as i16 as i32;
            match st.hit(x, y) {
                Hit::Launch(i) => launch(st, i),
                Hit::Tab(i) => {
                    st.active = i;
                    layout(st);
                    SetForegroundWindow(st.tabs[i].hwnd);
                }
                Hit::Close(i) => {
                    st.active = i;
                    layout(st);
                    PostMessageW(st.tabs[i].hwnd, 0x0010, 0, 0); // WM_CLOSE: приложение само спросит «сохранить?»
                }
                Hit::Crash => {
                    log(st, "УБИВАЮ СЕБЯ (TerminateProcess)");
                    TerminateProcess(GetCurrentProcess(), 99);
                }
                Hit::ToggleFrame => st.strip_frame = !st.strip_frame,
                Hit::Nothing => {}
            }
            InvalidateRect(st.main, null(), 0);
            Some(0)
        }
        0x0204 => {
            // WM_RBUTTONDOWN по табу — отцепить его
            let x = (lp & 0xFFFF) as i16 as i32;
            let y = ((lp >> 16) & 0xFFFF) as i16 as i32;
            if let Hit::Tab(i) | Hit::Close(i) = st.hit(x, y) {
                let t = st.tabs.remove(i);
                st.active = st.active.min(st.tabs.len().saturating_sub(1));
                detach(st, &t, st.s(40));
                layout(st);
                InvalidateRect(st.main, null(), 0);
            }
            Some(0)
        }
        0x02E0 => {
            // WM_DPICHANGED
            st.scale = (wp & 0xFFFF) as f32 / 96.0;
            DeleteObject(st.font);
            st.font = make_font(st.scale);
            let r = *(lp as *const RECT);
            SetWindowPos(st.main, null_mut(), r.l, r.t, r.r - r.l, r.b - r.t, SWP_NOZORDER | SWP_NOACTIVATE);
            InvalidateRect(st.main, null(), 0);
            Some(0)
        }
        0x0010 => {
            // WM_CLOSE: штатный выход — возвращаем окнам рамку и отпускаем их
            log(st, "ВЫХОД (WM_CLOSE)");
            detach_all(st);
            DestroyWindow(st.main);
            Some(0)
        }
        _ => None,
    }
}

extern "system" fn wndproc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    unsafe {
        if m == 0x0002 {
            PostQuitMessage(0);
            return 0;
        }
        // Реентерабельность: если состояние уже занято (вложенное сообщение), отдаём DefWindowProc.
        let r = STATE.with(|s| {
            let mut g = s.try_borrow_mut().ok()?;
            handle(g.as_mut()?, m, wp, lp)
        });
        r.unwrap_or_else(|| DefWindowProcW(h, m, wp, lp))
    }
}

fn main() {
    unsafe {
        SetProcessDpiAwarenessContext(-4isize as HANDLE); // PER_MONITOR_AWARE_V2
        let inst = GetModuleHandleW(null());
        let cls = w("wfoxd_probe3");
        RegisterClassW(&WNDCLASSW {
            style: 3,
            wndproc,
            cls_extra: 0,
            wnd_extra: 0,
            inst,
            icon: null_mut(),
            cursor: LoadCursorW(null_mut(), 32512 as *const u16),
            bg: null_mut(),
            menu: null(),
            name: cls.as_ptr(),
        });
        const CW_USEDEFAULT: i32 = 0x8000_0000_u32 as i32;
        let main = CreateWindowExW(0, cls.as_ptr(), w("wfoxd probe 3").as_ptr(), 0x00CF_0000 | 0x0200_0000, CW_USEDEFAULT, CW_USEDEFAULT, 1100, 700, null_mut(), null_mut(), inst, null_mut());
        let scale = GetDpiForWindow(main).max(96) as f32 / 96.0;
        SetWindowPos(main, null_mut(), 0, 0, (1100.0 * scale) as i32, (700.0 * scale) as i32, SWP_NOMOVE | SWP_NOZORDER);
        let cwd = std::env::var("USERPROFILE").or_else(|_| std::env::current_dir().map(|p| p.display().to_string())).unwrap_or_default();
        STATE.with(|s| {
            let st = State { main, scale, font: make_font(scale), tabs: Vec::new(), active: 0, pending: Vec::new(), status: String::new(), cwd, strip_frame: true, log_path: pick_log_path(), t_start: GetTickCount() };
            log(&st, &format!("===== wfoxd probe 3 старт, DPI={} =====", GetDpiForWindow(main)));
            *s.borrow_mut() = Some(st);
        });
        ShowWindow(main, 1);
        SetTimer(main, 1, 100, null());
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
