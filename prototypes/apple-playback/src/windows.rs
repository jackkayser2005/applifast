//! Temporary gate host. Every COM object lives on the dedicated STA thread.
use crate::protocol::{Command, validate_developer_token};
use keyring_core::api::CredentialStoreApi;
use serde_json::{Value, json};
use std::result::Result;
use std::{
    cell::{Cell, RefCell},
    io::{self, BufRead, Read},
    path::PathBuf,
    rc::Rc,
    sync::mpsc,
    thread,
};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::{Com::*, LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::*,
    },
    core::{Interface, PCWSTR, PWSTR, w},
};

const PAGE: &str = "https://applifast.invalid/index.html";
const SERVICE: &str = "local.applifast.playback-probe";
const WAKE: u32 = WM_APP + 1;
const AUTH_CLOSED: u32 = WM_APP + 2;
const AUTH_DISMISSED: u32 = WM_APP + 3;
const VIEW_CHANGED: u32 = WM_APP + 4;
const MAX_MESSAGE: usize = 1024 * 1024;

enum Output {
    Ready(u32),
    Bridge(Value),
    Error(&'static str),
    Input(String),
    Eof,
}
enum HostCommand {
    Dispatch(Command),
    Show(bool),
    Shutdown,
}

fn report(value: Value) {
    println!("{value}");
}
fn same_fields(input: &Value, canonical: &Value) -> bool {
    match (input, canonical) {
        (Value::Object(input), Value::Object(canonical)) => {
            input.len() == canonical.len()
                && input.iter().all(|(key, value)| {
                    canonical
                        .get(key)
                        .is_some_and(|expected| same_fields(value, expected))
                })
        }
        (Value::Array(input), Value::Array(canonical)) => {
            input.len() == canonical.len()
                && input
                    .iter()
                    .zip(canonical)
                    .all(|(value, expected)| same_fields(value, expected))
        }
        _ => true,
    }
}
fn parse_command(input: Value) -> Result<Command, &'static str> {
    let command: Command = serde_json::from_value(input.clone()).map_err(|_| "Invalid command.")?;
    command.validate().map_err(|_| "Invalid command.")?;
    let canonical = serde_json::to_value(&command).map_err(|_| "Invalid command.")?;
    if !same_fields(&input, &canonical) {
        return Err("Invalid command.");
    }
    Ok(command)
}
fn requested_command(value: &Value, session: u64) -> Option<Result<String, &'static str>> {
    if value.get("type").and_then(Value::as_str) != Some("command")
        || value.get("session").and_then(Value::as_u64) != Some(session)
    {
        return None;
    }
    Some((|| {
        let input = value.get("command").ok_or("Missing command.")?;
        let command = parse_command(input.clone())?;
        serde_json::to_string(&command).map_err(|_| "Invalid command.")
    })())
}
fn sanitized_event(value: &Value) -> Option<Value> {
    let kind = value.get("type")?.as_str()?;
    let fields: &[&str] = match kind {
        "ready" | "probe" | "report" => &[
            "version",
            "sdkVersion",
            "drm",
            "authorized",
            "storefront",
            "secureContext",
        ],
        "state" => &["status", "position", "duration", "index", "queueLength"],
        "signedOut" => &[],
        "library" => &["next"],
        _ => return None,
    };
    let mut result = json!({"type":kind,"session":value.get("session")?.as_u64()?});
    for field in fields {
        if let Some(data) = value.get(field) {
            let valid = match *field {
                "drm" | "authorized" | "secureContext" => data.is_boolean(),
                "status" | "position" | "duration" | "index" | "queueLength" => data.is_number(),
                "next" => {
                    data.is_null()
                        || data.as_str().is_some_and(|path| {
                            Command::Library {
                                next: Some(path.into()),
                            }
                            .validate()
                            .is_ok()
                        })
                }
                _ => data.as_str().is_some_and(|text| text.len() <= 256),
            };
            if valid {
                result[*field] = data.clone();
            }
        }
    }
    if kind == "library" {
        result["items"] = Value::Array(
            value
                .get("items")?
                .as_array()?
                .iter()
                .take(1000)
                .map(|item| {
                    let mut safe = json!({});
                    for field in [
                        "kind",
                        "id",
                        "title",
                        "artist",
                        "album",
                        "durationMs",
                        "catalogId",
                    ] {
                        if let Some(data) = item.get(field)
                            && (data.is_string()
                                || (field == "durationMs" && data.is_number())
                                || (field == "catalogId" && data.is_null()))
                        {
                            safe[field] = data.clone();
                        }
                    }
                    if let Some(params) = item.get("playParams") {
                        if params.is_null() {
                            safe["playParams"] = Value::Null;
                            return safe;
                        }
                        let mut safe_params = json!({});
                        for field in ["id", "kind", "isLibrary", "catalogId"] {
                            if let Some(data) = params.get(field)
                                && (data.is_string() || (field == "isLibrary" && data.is_boolean()))
                            {
                                safe_params[field] = data.clone();
                            }
                        }
                        safe["playParams"] = safe_params;
                    }
                    safe
                })
                .collect(),
        );
    }
    Some(result)
}
fn secret_entry(name: &str) -> Result<keyring_core::Entry, String> {
    windows_native_keyring_store::Store::new()
        .and_then(|store| store.build(SERVICE, name, None))
        .map_err(|_| "Windows Credential Manager is unavailable.".into())
}
fn read_secret(name: &str) -> Result<Option<String>, String> {
    match secret_entry(name)?.get_secret() {
        Ok(bytes) => String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "Stored credential is invalid.".into()),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(_) => Err("Cannot read Windows Credential Manager.".into()),
    }
}
fn write_secret(name: &str, value: &str) -> Result<(), String> {
    secret_entry(name)?
        .set_secret(value.as_bytes())
        .map_err(|_| "Cannot save Windows Credential Manager entry.".into())
}
fn delete_secret(name: &str) -> Result<(), String> {
    match secret_entry(name)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(_) => Err("Cannot clear Windows Credential Manager entry.".into()),
    }
}
fn profile_directory() -> Result<PathBuf, String> {
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable.")?)
            .join("Applifast/playback-probe"),
    )
}
fn revocation_marker() -> Result<PathBuf, String> {
    Ok(profile_directory()?.join("signed-out"))
}
fn mark_signed_out() -> Result<(), String> {
    std::fs::create_dir_all(profile_directory()?).map_err(|_| "Cannot persist sign-out marker.")?;
    std::fs::File::create(revocation_marker()?)
        .and_then(|file| file.sync_all())
        .map_err(|_| "Cannot persist sign-out marker.".into())
}
fn clear_revocation_marker() -> Result<(), String> {
    match std::fs::remove_file(revocation_marker()?) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Cannot clear sign-out marker.".into()),
    }
}

pub fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut self_check = false;
    let mut imported = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--self-check" => self_check = true,
            "--token-file" => {
                let path = args.next().ok_or("--token-file needs a path")?;
                let file =
                    std::fs::File::open(path).map_err(|_| "Cannot open developer token file.")?;
                let mut bytes = Vec::new();
                file.take(32769)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "Cannot read developer token file.")?;
                if bytes.len() > 32768 {
                    return Err("Developer token file exceeds 32 KiB.".into());
                }
                let token =
                    String::from_utf8(bytes).map_err(|_| "Developer token must be UTF-8.")?;
                validate_developer_token(token.trim())?;
                imported = Some(token.trim().to_owned());
            }
            _ => {
                return Err(
                    "Usage: applifast-playback-probe [--token-file PATH] [--self-check]".into(),
                );
            }
        }
    }
    let revoked = !self_check
        && revocation_marker()?
            .try_exists()
            .map_err(|_| "Cannot read sign-out state.")?;
    let (developer, user) = if self_check {
        (String::new(), None)
    } else {
        if let Some(token) = imported {
            write_secret("developer-token", &token)?;
        }
        let developer = read_secret("developer-token")?
            .ok_or("Import your developer token with --token-file PATH.")?;
        validate_developer_token(&developer)?;
        (
            developer,
            if revoked {
                None
            } else {
                read_secret("music-user-token")?
            },
        )
    };
    let (out_tx, out_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let sta_tx = out_tx.clone();
    let host = thread::spawn(move || {
        if let Err(error) = host_loop(cmd_rx, sta_tx.clone(), developer, user, self_check, revoked)
        {
            let code = error
                .downcast_ref::<windows::core::Error>()
                .map(|error| error.code().0)
                .or_else(|| {
                    error
                        .downcast_ref::<webview2_com::Error>()
                        .and_then(|error| match error {
                            webview2_com::Error::WindowsError(error) => Some(error.code().0),
                            _ => None,
                        })
                });
            if let Some(code) = code {
                report(json!({"type":"hostErrorCode","hresult":format!("0x{:08X}", code as u32)}));
            }
            let _ = sta_tx.send(Output::Error(
                "WebView2 host failed. Install or repair the Evergreen runtime.",
            ));
        }
    });
    let mut thread_id = None;
    let mut epoch = 1u64;
    let mut credentials_failed = false;
    let mut host_failed = false;
    for event in out_rx {
        match event {
            Output::Ready(id) => {
                thread_id = Some(id);
                report(json!({"type":"hostReady","pid":std::process::id(),"selfCheck":self_check}));
                let input_tx = out_tx.clone();
                thread::spawn(move || {
                    let mut input = io::stdin().lock();
                    loop {
                        let mut line = Vec::new();
                        let result = (&mut input)
                            .take((MAX_MESSAGE + 1) as u64)
                            .read_until(b'\n', &mut line);
                        if !matches!(result, Ok(size) if size > 0) {
                            break;
                        }
                        if line.len() > MAX_MESSAGE {
                            let _ = input_tx
                                .send(Output::Error("Command exceeds 1 MiB; input closed."));
                            break;
                        }
                        match String::from_utf8(line) {
                            Ok(line) => {
                                if input_tx.send(Output::Input(line)).is_err() {
                                    return;
                                }
                            }
                            Err(_) => {
                                let _ = input_tx.send(Output::Error("Command must be UTF-8."));
                            }
                        }
                    }
                    let _ = input_tx.send(Output::Eof);
                });
            }
            Output::Input(line) => {
                let command = match line.trim() {
                    "show" => HostCommand::Show(true),
                    "hide" => HostCommand::Show(false),
                    _ => match serde_json::from_str::<Value>(&line)
                        .map_err(|_| "Invalid command.")
                        .and_then(parse_command)
                    {
                        Ok(Command::Shutdown) => HostCommand::Shutdown,
                        Ok(command) => {
                            if let Err(error) = command.validate() {
                                report(json!({"type":"error","message":error}));
                                continue;
                            }
                            if matches!(command, Command::SignOut) {
                                epoch += 1;
                                if let Err(error) = mark_signed_out() {
                                    report(json!({"type":"error","message":error}));
                                    credentials_failed = true;
                                }
                                if let Err(error) = delete_secret("music-user-token") {
                                    report(json!({"type":"error","message":error}));
                                    credentials_failed = true;
                                }
                            }
                            HostCommand::Dispatch(command)
                        }
                        Err(_) => {
                            report(json!({"type":"error","message":"Invalid command."}));
                            continue;
                        }
                    },
                };
                let shutdown = matches!(command, HostCommand::Shutdown);
                cmd_tx.send(command).map_err(|_| "Playback host stopped.")?;
                if let Some(id) = thread_id {
                    unsafe {
                        PostThreadMessageW(id, WAKE, WPARAM(0), LPARAM(0))
                            .map_err(|_| "Playback host wake failed.")?;
                    }
                }
                if shutdown {
                    break;
                }
            }
            Output::Bridge(value) => {
                if value.get("session").and_then(Value::as_u64) != Some(epoch) {
                    continue;
                }
                match value.get("type").and_then(Value::as_str) {
                    Some("authorized") => {
                        if let Some(token) = value
                            .get("token")
                            .and_then(Value::as_str)
                            .filter(|token| !token.is_empty() && token.len() <= 32768)
                        {
                            write_secret("music-user-token", token)?;
                            clear_revocation_marker()?;
                            report(json!({"type":"authorized","session":epoch}));
                        }
                    }
                    Some("error") => {
                        let message = if value.get("code").and_then(Value::as_str)
                            == Some("AUTH_CLOSED")
                        {
                            "Apple authorization window was closed. Authorize again to retry."
                        } else {
                            "MusicKit rejected an operation. Check authorization, subscription, connection, or the selected item."
                        };
                        report(json!({"type":"error","session":epoch,"message":message}));
                    }
                    Some("ready" | "library" | "state" | "probe" | "report" | "signedOut") => {
                        if let Some(value) = sanitized_event(&value) {
                            report(value);
                        }
                    }
                    _ => {}
                }
            }
            Output::Error(message) => {
                report(json!({"type":"error","message":message}));
                host_failed = true;
                break;
            }
            Output::Eof => {
                let _ = cmd_tx.send(HostCommand::Shutdown);
                if let Some(id) = thread_id {
                    unsafe {
                        let _ = PostThreadMessageW(id, WAKE, WPARAM(0), LPARAM(0));
                    }
                }
                break;
            }
        }
    }
    // Wake on errors too, so the STA cannot outlive the console.
    let _ = cmd_tx.send(HostCommand::Shutdown);
    if let Some(id) = thread_id {
        unsafe {
            let _ = PostThreadMessageW(id, WAKE, WPARAM(0), LPARAM(0));
        }
    }
    host.join()
        .map_err(|_| "Playback host thread stopped unexpectedly.")?;
    if credentials_failed {
        return Err("Sign-out could not clear saved credentials. Resolve Credential Manager access before restarting.".into());
    }
    if host_failed {
        return Err("Probe did not finish successfully.".into());
    }
    Ok(())
}

fn apple_auth_url(uri: &str) -> bool {
    url::Url::parse(uri).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port_or_known_default() == Some(443)
            && url
                .host_str()
                .is_some_and(|host| host == "apple.com" || host.ends_with(".apple.com"))
    })
}
fn bridge_origin(uri: &str) -> bool {
    uri == PAGE
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
unsafe fn com_string(
    read: impl FnOnce(*mut PWSTR) -> windows::core::Result<()>,
) -> windows::core::Result<String> {
    let mut value = PWSTR::null();
    read(&mut value)?;
    Ok(CoTaskMemPWSTR::from(value).to_string())
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_SIZE {
        unsafe {
            let _ = PostThreadMessageW(
                GetCurrentThreadId(),
                VIEW_CHANGED,
                WPARAM(hwnd.0 as usize),
                LPARAM((wp.0 != SIZE_MINIMIZED as usize) as isize),
            );
        }
    }
    if msg == WM_CLOSE {
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
            let _ = PostThreadMessageW(
                GetCurrentThreadId(),
                AUTH_CLOSED,
                WPARAM(hwnd.0 as usize),
                LPARAM(0),
            );
        }
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}
unsafe fn create_window() -> windows::core::Result<HWND> {
    unsafe {
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let class = WNDCLASSW {
            hInstance: instance,
            lpfnWndProc: Some(window_proc),
            lpszClassName: w!("ApplifastPlaybackProbe"),
            ..Default::default()
        };
        RegisterClassW(&class);
        CreateWindowExW(
            Default::default(),
            w!("ApplifastPlaybackProbe"),
            w!("Applifast Apple Music authorization"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800,
            700,
            None,
            None,
            Some(instance),
            None,
        )
    }
}
fn controller(
    environment: &ICoreWebView2Environment,
    hwnd: HWND,
) -> Result<ICoreWebView2Controller, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let environment = environment.clone();
    CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            environment
                .CreateCoreWebView2Controller(hwnd, &handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |result, controller| {
            result?;
            let _ = tx.send(controller.ok_or_else(windows::core::Error::from_thread));
            Ok(())
        }),
    )?;
    let controller = rx.recv()??;
    unsafe {
        controller.SetBounds(RECT {
            left: 0,
            top: 0,
            right: 780,
            bottom: 660,
        })?;
        controller.SetIsVisible(true)?;
    }
    Ok(controller)
}

fn set_view_visibility(
    controller: &ICoreWebView2Controller,
    webview: &ICoreWebView2,
    visible: bool,
) -> windows::core::Result<()> {
    unsafe {
        controller.SetIsVisible(visible)?;
        webview
            .cast::<ICoreWebView2_19>()?
            .SetMemoryUsageTargetLevel(if visible {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
            } else {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
            })
    }
}

fn restrict(webview: &ICoreWebView2, auth: bool) -> windows::core::Result<()> {
    unsafe {
        let settings = webview.Settings()?;
        settings.SetAreDevToolsEnabled(false)?;
        settings.SetAreDefaultContextMenusEnabled(false)?;
        settings.SetIsWebMessageEnabled(!auth)?;
        let mut token = 0;
        webview.add_NavigationStarting(
            &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let uri = com_string(|out| args.Uri(out))?;
                    if !(if auth {
                        apple_auth_url(&uri) || uri == "about:blank"
                    } else {
                        bridge_origin(&uri)
                    }) {
                        args.SetCancel(true)?;
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;
        webview.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut kind = COREWEBVIEW2_PERMISSION_KIND_UNKNOWN_PERMISSION;
                    args.PermissionKind(&mut kind)?;
                    let uri = com_string(|out| args.Uri(out))?;
                    let own_origin = url::Url::parse(&uri).is_ok_and(|url| {
                        url.origin().ascii_serialization() == "https://applifast.invalid"
                    });
                    args.SetState(
                        if !auth && own_origin && kind == COREWEBVIEW2_PERMISSION_KIND_AUTOPLAY {
                            COREWEBVIEW2_PERMISSION_STATE_ALLOW
                        } else {
                            COREWEBVIEW2_PERMISSION_STATE_DENY
                        },
                    )?;
                }
                Ok(())
            })),
            &mut token,
        )?;
        webview.cast::<ICoreWebView2_4>()?.add_DownloadStarting(
            &DownloadStartingEventHandler::create(Box::new(|_, args| {
                if let Some(args) = args {
                    args.SetCancel(true)?;
                }
                Ok(())
            })),
            &mut token,
        )?;
    }
    Ok(())
}

fn host_loop(
    rx: mpsc::Receiver<HostCommand>,
    output: mpsc::Sender<Output>,
    developer: String,
    user: Option<String>,
    self_check: bool,
    revoked: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    }
    let data = profile_directory()?;
    std::fs::create_dir_all(&data)?;
    let path = wide(&data.to_string_lossy());
    let (tx, env_rx) = mpsc::channel();
    CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            CreateCoreWebView2EnvironmentWithOptions(
                PCWSTR::null(),
                PCWSTR(path.as_ptr()),
                None,
                &handler,
            )
            .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |result, environment| {
            result?;
            let _ = tx.send(environment.ok_or_else(windows::core::Error::from_thread));
            Ok(())
        }),
    )?;
    let environment = env_rx.recv()??;
    let hwnd = unsafe { create_window()? };
    let main_controller = controller(&environment, hwnd)?;
    let webview = unsafe { main_controller.CoreWebView2()? };
    set_view_visibility(&main_controller, &webview, false)?;
    if revoked {
        let (clear_tx, clear_rx) = mpsc::channel();
        unsafe {
            webview
                .cast::<ICoreWebView2_13>()?
                .Profile()?
                .cast::<ICoreWebView2Profile2>()?
                .ClearBrowsingDataAll(&ClearBrowsingDataCompletedHandler::create(Box::new(
                    move |result| {
                        let _ = clear_tx.send(result);
                        Ok(())
                    },
                )))?;
        }
        webview2_com::wait_with_pump(clear_rx)??;
    }
    restrict(&webview, false)?;
    let folder = wide(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("web")
            .to_string_lossy(),
    );
    if !self_check {
        unsafe {
            webview
                .cast::<ICoreWebView2_3>()?
                .SetVirtualHostNameToFolderMapping(
                    w!("applifast.invalid"),
                    PCWSTR(folder.as_ptr()),
                    COREWEBVIEW2_HOST_RESOURCE_ACCESS_KIND_DENY_CORS,
                )?;
        }
    }
    let popups: Rc<RefCell<Vec<(HWND, ICoreWebView2Controller)>>> =
        Rc::new(RefCell::new(Vec::new()));
    let generation = Rc::new(Cell::new(1u64));
    let page_ready = Rc::new(Cell::new(self_check));
    let authorized_popup = Rc::new(Cell::new(false));
    let mut token = 0;
    let bootstrap =
        serde_json::to_string(&json!({"developerToken":developer,"userToken":user,"session":1}))?;
    let init = wide(&format!(
        "if(location.href === {page}) document.addEventListener('DOMContentLoaded', () => window.applifast.bootstrap({bootstrap}));",
        page = serde_json::to_string(PAGE)?
    ));
    unsafe {
        webview.AddScriptToExecuteOnDocumentCreated(
            PCWSTR(init.as_ptr()),
            &AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(|_, _| Ok(()))),
        )?;
        let event_output = output.clone();
        let event_generation = generation.clone();
        let auth_popups = popups.clone();
        let permission = authorized_popup.clone();
        let event_ready = page_ready.clone();
        webview.add_WebMessageReceived(
            &WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    if !bridge_origin(&com_string(|out| args.Source(out))?) {
                        return Ok(());
                    }
                    let text = com_string(|out| args.WebMessageAsJson(out))?;
                    if text.len() > MAX_MESSAGE {
                        return Ok(());
                    }
                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                        if value.get("session").and_then(Value::as_u64)
                            != Some(event_generation.get())
                        {
                            return Ok(());
                        }
                        if let Some(command) = requested_command(&value, event_generation.get()) {
                            match command {
                                Ok(command) => { let _ = event_output.send(Output::Input(command)); }
                                Err(_) => { let _ = event_output.send(Output::Bridge(json!({"type":"error","session":event_generation.get(),"code":"INVALID_COMMAND"}))); }
                            }
                            return Ok(());
                        }
                        if value.get("type").and_then(Value::as_str) == Some("ready") {
                            event_ready.set(true);
                            let _ = PostThreadMessageW(
                                GetCurrentThreadId(),
                                WAKE,
                                WPARAM(0),
                                LPARAM(0),
                            );
                        }
                        if matches!(
                            value.get("type").and_then(Value::as_str),
                            Some("authorized" | "signedOut")
                        ) {
                            permission.set(false);
                            for (window, controller) in auth_popups.borrow_mut().drain(..) {
                                let _ = controller.Close();
                                let _ = DestroyWindow(window);
                            }
                        }
                        let _ = event_output.send(Output::Bridge(value));
                    }
                }
                Ok(())
            })),
            &mut token,
        )?;
        let popup_environment = environment.clone();
        let popup_collection = popups.clone();
        let popup_permission = authorized_popup.clone();
        let popup_output = output.clone();
        webview.add_NewWindowRequested(
            &NewWindowRequestedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                args.SetHandled(true)?;
                let uri = com_string(|out| args.Uri(out))?;
                if !popup_permission.get() || !apple_auth_url(&uri) {
                    return Ok(());
                }
                let deferral = args.GetDeferral()?;
                let popup = create_window()?;
                let collection = popup_collection.clone();
                let errors = popup_output.clone();
                let handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(
                    move |result, created| {
                        let configured = (|| -> windows::core::Result<()> {
                            result?;
                            let created = created.ok_or_else(windows::core::Error::from_thread)?;
                            created.SetBounds(RECT {
                                left: 0,
                                top: 0,
                                right: 780,
                                bottom: 660,
                            })?;
                            created.SetIsVisible(true)?;
                            let child = created.CoreWebView2()?;
                            restrict(&child, true)?;
                            child.add_WindowCloseRequested(
                                &WindowCloseRequestedEventHandler::create(Box::new(move |_, _| {
                                    let _ = PostThreadMessageW(
                                        GetCurrentThreadId(),
                                        AUTH_DISMISSED,
                                        WPARAM(popup.0 as usize),
                                        LPARAM(0),
                                    );
                                    Ok(())
                                })),
                                &mut 0,
                            )?;
                            args.SetNewWindow(&child)?;
                            child.add_NewWindowRequested(
                                &NewWindowRequestedEventHandler::create(Box::new(|_, args| {
                                    if let Some(args) = args {
                                        args.SetHandled(true)?;
                                    }
                                    Ok(())
                                })),
                                &mut 0,
                            )?;
                            collection.borrow_mut().push((popup, created));
                            let _ = ShowWindow(popup, SW_SHOW);
                            Ok(())
                        })();
                        deferral.Complete()?;
                        if configured.is_err() {
                            let _ = errors
                                .send(Output::Error("Apple authorization popup could not open."));
                        }
                        Ok(())
                    },
                ));
                popup_environment.CreateCoreWebView2Controller(popup, &handler)?;
                Ok(())
            })),
            &mut token,
        )?;
        if self_check {
            let browser = com_string(|out| environment.BrowserVersionString(out))?;
            report(json!({"type":"runtime","version":browser,"pid":std::process::id()}));
        } else {
            webview.Navigate(w!("https://applifast.invalid/index.html"))?;
        }
    }
    let id = unsafe { GetCurrentThreadId() };
    let _ = output.send(Output::Ready(id));
    let mut message = MSG::default();
    let mut pending = Vec::new();
    'pump: loop {
        unsafe {
            let result = GetMessageW(&mut message, None, 0, 0).0;
            if result < 0 {
                return Err(windows::core::Error::from_thread().into());
            }
            if result == 0 {
                break;
            }
            if message.message == VIEW_CHANGED && message.wParam.0 == hwnd.0 as usize {
                let visible = message.lParam.0 != 0 && IsWindowVisible(hwnd).as_bool();
                set_view_visibility(&main_controller, &webview, visible)?;
                let mut bounds = RECT::default();
                GetClientRect(hwnd, &mut bounds)?;
                main_controller.SetBounds(bounds)?;
            } else if matches!(message.message, AUTH_CLOSED | AUTH_DISMISSED) {
                let mut windows = popups.borrow_mut();
                if let Some(index) = windows
                    .iter()
                    .position(|(window, _)| window.0 as usize == message.wParam.0)
                {
                    let (window, controller) = windows.remove(index);
                    let _ = controller.Close();
                    let _ = DestroyWindow(window);
                    authorized_popup.set(false);
                    if message.message == AUTH_CLOSED {
                        let _ = output.send(Output::Bridge(
                            json!({"type":"error","session":generation.get(),"code":"AUTH_CLOSED"}),
                        ));
                    }
                } else if message.wParam.0 == hwnd.0 as usize {
                    set_view_visibility(&main_controller, &webview, false)?;
                }
            } else if message.message == WAKE {
                let mut commands = if page_ready.get() {
                    std::mem::take(&mut pending)
                } else {
                    Vec::new()
                };
                commands.extend(rx.try_iter());
                for command in commands {
                    match command {
                        HostCommand::Shutdown => break 'pump,
                        HostCommand::Show(show) => {
                            set_view_visibility(&main_controller, &webview, show)?;
                            let _ = ShowWindow(hwnd, if show { SW_SHOW } else { SW_HIDE });
                        }
                        HostCommand::Dispatch(command) => {
                            if !page_ready.get() {
                                if pending.len() >= 100 {
                                    let _ = output.send(Output::Error("MusicKit has not initialized; pending command limit reached."));
                                    break 'pump;
                                }
                                pending.push(HostCommand::Dispatch(command));
                                continue;
                            }
                            if matches!(command, Command::Authorize) {
                                authorized_popup.set(true);
                            }
                            if matches!(command, Command::SignOut) {
                                generation.set(generation.get() + 1);
                                authorized_popup.set(false);
                                for (window, controller) in popups.borrow_mut().drain(..) {
                                    let _ = controller.Close();
                                    let _ = DestroyWindow(window);
                                }
                                let profile = webview
                                    .cast::<ICoreWebView2_13>()?
                                    .Profile()?
                                    .cast::<ICoreWebView2Profile2>()?;
                                profile.ClearBrowsingDataAll(
                                    &ClearBrowsingDataCompletedHandler::create(Box::new(
                                        |result| {
                                            report(json!({"type":"profileCleared","success":result.is_ok()}));
                                            result
                                        },
                                    )),
                                )?;
                            }
                            let json = serde_json::to_string(&command)?;
                            let script = wide(&format!("window.applifast.dispatch({json})"));
                            webview.ExecuteScript(
                                PCWSTR(script.as_ptr()),
                                &ExecuteScriptCompletedHandler::create(Box::new(|result, _| {
                                    result
                                })),
                            )?;
                        }
                    }
                }
            } else {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
    unsafe {
        for (window, controller) in popups.borrow_mut().drain(..) {
            let _ = controller.Close();
            let _ = DestroyWindow(window);
        }
        main_controller.Close()?;
        DestroyWindow(hwnd)?;
        CoUninitialize();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bridge_and_popup_boundaries() {
        assert!(bridge_origin(PAGE));
        assert!(!bridge_origin("https://applifast.invalid/index.html?x"));
        assert!(!bridge_origin("https://applifast.invalid.evil/index.html"));
        assert!(apple_auth_url(
            "https://idmsa.apple.com/appleauth/auth/authorize"
        ));
        for uri in [
            "http://apple.com",
            "https://apple.com.evil",
            "https://evilapple.com",
            "https://evil@apple.com",
            "https://apple.com:8443",
            "file:///apple.com",
        ] {
            assert!(!apple_auth_url(uri));
        }
    }
    #[test]
    fn events_never_echo_arbitrary_credential_fields() {
        let value = sanitized_event(&json!({"type":"state","session":1,"status":2,"token":"SECRET","unexpected":{"token":"SECRET"}})).unwrap();
        assert_eq!(value, json!({"type":"state","session":1,"status":2}));
        let value = sanitized_event(&json!({"type":"library","session":1,"next":null,"items":[{"kind":"library","id":"i.1","token":"SECRET","playParams":{"id":"i.1","isLibrary":true,"token":"SECRET"}}]})).unwrap();
        assert!(!value.to_string().contains("SECRET"));
        assert!(
            sanitized_event(&json!({"type":"authorized","session":1,"token":"SECRET"})).is_none()
        );
        let value = sanitized_event(&json!({"type":"probe","session":1,"sdkVersion":"3","drm":{"token":"SECRET"},"keySystems":{"token":"SECRET"}})).unwrap();
        assert_eq!(value, json!({"type":"probe","session":1,"sdkVersion":"3"}));
        let value = sanitized_event(
            &json!({"type":"library","session":1,"items":[{"id":"i.1","playParams":null}]}),
        )
        .unwrap();
        assert!(value["items"][0]["playParams"].is_null());
    }
    #[test]
    fn page_commands_use_the_validated_native_command_path() {
        let command = requested_command(
            &json!({"type":"command","session":4,"command":{"type":"signOut"}}),
            4,
        )
        .unwrap()
        .unwrap();
        assert_eq!(command, r#"{"type":"signOut"}"#);
        assert!(
            requested_command(
                &json!({"type":"command","session":3,"command":{"type":"authorize"}}),
                4
            )
            .is_none()
        );
        for command in [
            json!({"type":"volume","value":2}),
            json!({"type":"unsupported"}),
            json!({"type":"authorize","token":"SECRET"}),
        ] {
            assert!(
                requested_command(&json!({"type":"command","session":4,"command":command}), 4)
                    .unwrap()
                    .is_err()
            );
        }
        assert!(parse_command(json!({"type":"library"})).is_err());
        assert!(parse_command(json!({"type":"library","next":null})).is_ok());
        assert!(parse_command(json!({"type":"volume","value":0})).is_ok());
        assert!(
            parse_command(json!({"type":"play","index":0,"items":[{"kind":"library","id":"i.1"}]}))
                .is_err()
        );
    }
    #[test]
    #[ignore = "Requires Windows Credential Manager; deletes its own isolated dummy entry"]
    fn native_store_round_trip() {
        let name = format!("test-{}", std::process::id());
        write_secret(&name, "dummy-probe-grant").unwrap();
        assert_eq!(
            read_secret(&name).unwrap().as_deref(),
            Some("dummy-probe-grant")
        );
        delete_secret(&name).unwrap();
        assert!(read_secret(&name).unwrap().is_none());
    }
}
