use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteTreeW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_DWORD, REG_NONE,
    REG_SAM_FLAGS, REG_SZ,
};
use windows::Win32::UI::Shell::{
    ApplicationAssociationRegistration, IApplicationAssociationRegistration, SHChangeNotify,
    AL_EFFECTIVE, AT_FILEEXTENSION, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
};

const APP_NAME: &str = "Native Reader";
const PROG_ID: &str = "NativeReader.TextFile";
const CAPABILITIES_PATH: &str = r"Software\NativeReader\Capabilities";
const ASSOCIATIONS_PATH: &str = r"Software\NativeReader\Associations";
const APPLICATION_PATH: &str = r"Software\Classes\Applications\native-reader.exe";

pub const TEXT_EXTENSIONS: &[&str] = &[
    ".md",
    ".markdown",
    ".mdown",
    ".mkd",
    ".mdx",
    ".html",
    ".htm",
    ".xhtml",
    ".xml",
    ".svg",
    ".txt",
    ".log",
    ".csv",
    ".tsv",
    ".json",
    ".jsonc",
    ".yaml",
    ".yml",
    ".toml",
    ".ini",
    ".cfg",
    ".conf",
    ".config",
    ".properties",
    ".rs",
    ".c",
    ".h",
    ".cc",
    ".cpp",
    ".cxx",
    ".hpp",
    ".hxx",
    ".hh",
    ".cs",
    ".java",
    ".kt",
    ".kts",
    ".scala",
    ".js",
    ".jsx",
    ".mjs",
    ".cjs",
    ".ts",
    ".tsx",
    ".mts",
    ".cts",
    ".py",
    ".pyw",
    ".pyi",
    ".go",
    ".sql",
    ".sh",
    ".bash",
    ".zsh",
    ".fish",
    ".ps1",
    ".psm1",
    ".psd1",
    ".css",
    ".scss",
    ".sass",
    ".less",
    ".lua",
    ".bat",
    ".cmd",
    ".rb",
    ".php",
    ".pl",
    ".pm",
    ".swift",
    ".dart",
    ".r",
    ".asm",
    ".s",
    ".vb",
    ".vbs",
    ".tex",
    ".rst",
    ".graphql",
    ".gql",
    ".proto",
    ".vue",
    ".svelte",
    ".cshtml",
    ".razor",
    ".xaml",
    ".csproj",
    ".vcxproj",
    ".props",
    ".targets",
    ".sln",
];

pub const RECOMMENDED_EXTENSIONS: &[&str] = &[
    ".md",
    ".markdown",
    ".mdown",
    ".mkd",
    ".mdx",
    ".html",
    ".htm",
    ".xhtml",
    ".txt",
    ".log",
    ".csv",
    ".tsv",
    ".json",
    ".jsonc",
    ".yaml",
    ".yml",
    ".toml",
    ".ini",
    ".xml",
    ".svg",
    ".rs",
    ".c",
    ".h",
    ".cc",
    ".cpp",
    ".hpp",
    ".cs",
    ".java",
    ".kt",
    ".js",
    ".jsx",
    ".mjs",
    ".ts",
    ".tsx",
    ".py",
    ".go",
    ".sql",
    ".sh",
    ".ps1",
    ".bat",
    ".cmd",
    ".css",
    ".scss",
    ".less",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssociationMode {
    All,
    Recommended,
    Custom,
    None,
}

impl AssociationMode {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "all" => Some(Self::All),
            "recommended" | "default" => Some(Self::Recommended),
            "custom" => Some(Self::Custom),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

pub struct AssociationSelection {
    pub mode: AssociationMode,
    pub extensions: Vec<&'static str>,
    pub set_default: bool,
}

impl AssociationSelection {
    fn load() -> Self {
        let mode = read_sz(ASSOCIATIONS_PATH, Some("Mode"))
            .and_then(|value| AssociationMode::parse(&value))
            .unwrap_or(AssociationMode::All);
        let set_default = read_dword(ASSOCIATIONS_PATH, Some("SetDefault"))
            .map(|value| value != 0)
            .unwrap_or(true);
        let extensions = match mode {
            AssociationMode::All => TEXT_EXTENSIONS.to_vec(),
            AssociationMode::Recommended => RECOMMENDED_EXTENSIONS.to_vec(),
            AssociationMode::Custom => read_sz(ASSOCIATIONS_PATH, Some("Extensions"))
                .map(|value| parse_extension_list(&value))
                .unwrap_or_default(),
            AssociationMode::None => Vec::new(),
        };
        Self {
            mode,
            extensions,
            set_default: set_default && mode != AssociationMode::None,
        }
    }
}

#[derive(Debug)]
pub struct AssociationReport {
    pub protected_extensions: Vec<&'static str>,
}

impl AssociationReport {
    pub fn needs_user_confirmation(&self) -> bool {
        !self.protected_extensions.is_empty()
    }
}

pub fn ensure_registered() -> Result<AssociationReport, String> {
    let selection = AssociationSelection::load();
    let exe = std::env::current_exe().map_err(|error| format!("无法获取程序路径：{error}"))?;
    let icon = crate::icon::install();
    register_keys(&exe, icon.as_deref(), &selection.extensions)?;
    let protected_extensions = if selection.set_default {
        set_defaults(&selection.extensions)
    } else {
        Vec::new()
    };
    notify_associations_changed();
    Ok(AssociationReport {
        protected_extensions,
    })
}

pub fn ensure_registered_if_needed() -> Result<AssociationReport, String> {
    let exe = std::env::current_exe().map_err(|error| format!("无法获取程序路径：{error}"))?;
    if registration_matches(&exe) {
        return Ok(AssociationReport {
            protected_extensions: Vec::new(),
        });
    }
    ensure_registered()
}

pub fn apply_configured() -> Result<AssociationReport, String> {
    ensure_registered()
}

pub fn remove_registered() -> Result<(), String> {
    for extension in TEXT_EXTENSIONS {
        remove_extension(extension);
    }
    delete_tree(&format!(r"Software\Classes\{PROG_ID}"));
    delete_tree(APPLICATION_PATH);
    delete_tree(CAPABILITIES_PATH);
    delete_tree(ASSOCIATIONS_PATH);
    delete_value(r"Software\RegisteredApplications", Some(APP_NAME));
    notify_associations_changed();
    Ok(())
}

pub fn defaults_prompted() -> bool {
    read_dword(ASSOCIATIONS_PATH, Some("Prompted"))
        .map(|value| value != 0)
        .unwrap_or(false)
}

pub fn mark_defaults_prompted() {
    let _ = with_key(ASSOCIATIONS_PATH, |key| set_dword(key, "Prompted", 1));
}

pub fn open_default_apps() {
    let target = "ms-settings:defaultapps?registeredAppUser=Native%20Reader";
    let operation: Vec<u16> = "open\0".encode_utf16().collect();
    let target: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = windows::Win32::UI::Shell::ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
    }
}

fn register_keys(exe: &Path, icon: Option<&Path>, selected: &[&'static str]) -> Result<(), String> {
    let exe = exe.to_string_lossy();
    let command = format!("\"{exe}\" \"%1\"");
    let icon_value = icon
        .map(|path| format!("\"{}\",0", path.to_string_lossy()))
        .unwrap_or_else(|| format!("\"{exe}\",0"));

    let prog_id = format!(r"Software\Classes\{PROG_ID}");
    with_key(&prog_id, |key| {
        set_sz(key, None, "文本 / 源代码文件")?;
        set_sz(key, Some("FriendlyTypeName"), "文本 / 源代码文件")
    })?;
    with_key(&format!(r"{prog_id}\DefaultIcon"), |key| {
        set_sz(key, None, &icon_value)
    })?;
    with_key(&format!(r"{prog_id}\shell\open\command"), |key| {
        set_sz(key, None, &command)
    })?;

    with_key(APPLICATION_PATH, |key| {
        set_sz(key, Some("FriendlyAppName"), APP_NAME)
    })?;
    with_key(&format!(r"{APPLICATION_PATH}\DefaultIcon"), |key| {
        set_sz(key, None, &icon_value)
    })?;
    with_key(&format!(r"{APPLICATION_PATH}\shell\open\command"), |key| {
        set_sz(key, None, &command)
    })?;

    with_key(CAPABILITIES_PATH, |key| {
        set_sz(key, Some("ApplicationName"), APP_NAME)?;
        set_sz(
            key,
            Some("ApplicationDescription"),
            "极简、原生、快速的 Markdown / 代码 / 文本查看器",
        )?;
        set_sz(key, Some("ApplicationIcon"), &icon_value)
    })?;
    delete_tree(&format!(r"{CAPABILITIES_PATH}\FileAssociations"));
    with_key(&format!(r"{CAPABILITIES_PATH}\FileAssociations"), |key| {
        for extension in selected {
            set_sz(key, Some(extension), PROG_ID)?;
        }
        Ok(())
    })?;
    with_key(r"Software\RegisteredApplications", |key| {
        set_sz(key, Some(APP_NAME), CAPABILITIES_PATH)
    })?;

    for extension in TEXT_EXTENSIONS {
        if selected.contains(extension) {
            register_extension(extension)?;
        } else {
            remove_extension(extension);
        }
    }
    Ok(())
}

fn registration_matches(exe: &Path) -> bool {
    let expected = format!("\"{}\" \"%1\"", exe.to_string_lossy());
    let command = read_sz(&format!(r"{APPLICATION_PATH}\shell\open\command"), None);
    let registered = read_sz(r"Software\RegisteredApplications", Some(APP_NAME));
    registered.as_deref() == Some(CAPABILITIES_PATH)
        && command
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(&expected))
}

fn register_extension(extension: &str) -> Result<(), String> {
    let extension_path = format!(r"Software\Classes\{extension}");
    with_key(&extension_path, |key| set_sz(key, None, PROG_ID))?;
    with_key(&format!(r"{extension_path}\OpenWithProgids"), |key| {
        set_none(key, PROG_ID)
    })?;
    with_key(&format!(r"{APPLICATION_PATH}\SupportedTypes"), |key| {
        set_none(key, extension)
    })?;
    Ok(())
}

fn remove_extension(extension: &str) {
    let extension_path = format!(r"Software\Classes\{extension}");
    if read_sz(&extension_path, None).as_deref() == Some(PROG_ID) {
        delete_value(&extension_path, None);
    }
    delete_value(&format!(r"{extension_path}\OpenWithProgids"), Some(PROG_ID));
    delete_value(
        &format!(r"{APPLICATION_PATH}\SupportedTypes"),
        Some(extension),
    );
}

fn set_defaults(extensions: &[&'static str]) -> Vec<&'static str> {
    if extensions.is_empty() {
        return Vec::new();
    }
    unsafe {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let result = (|| -> Result<Vec<&'static str>, String> {
            let registration: IApplicationAssociationRegistration = CoCreateInstance(
                &ApplicationAssociationRegistration,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .map_err(|error| format!("无法打开系统关联接口：{error}"))?;
            let app_name: Vec<u16> = APP_NAME.encode_utf16().chain(std::iter::once(0)).collect();
            let app_name = PCWSTR(app_name.as_ptr());
            let _ = registration.SetAppAsDefaultAll(app_name);

            let mut protected = Vec::new();
            for extension in extensions {
                let ext: Vec<u16> = extension.encode_utf16().chain(std::iter::once(0)).collect();
                let is_default = registration
                    .QueryAppIsDefault(
                        PCWSTR(ext.as_ptr()),
                        AT_FILEEXTENSION,
                        AL_EFFECTIVE,
                        app_name,
                    )
                    .map(|value| value.as_bool())
                    .unwrap_or(false);
                if !is_default || has_foreign_user_choice(extension) {
                    protected.push(*extension);
                }
            }
            Ok(protected)
        })();
        if initialized.is_ok() {
            CoUninitialize();
        }
        result.unwrap_or_else(|_| extensions.to_vec())
    }
}

fn has_foreign_user_choice(extension: &str) -> bool {
    let path = format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\{extension}\UserChoice"
    );
    read_sz(&path, Some("ProgId"))
        .map(|value| {
            let value = value.trim();
            !value.eq_ignore_ascii_case(PROG_ID)
                && !value.eq_ignore_ascii_case(r"Applications\native-reader.exe")
        })
        .unwrap_or(false)
}

fn notify_associations_changed() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

fn parse_extension_list(value: &str) -> Vec<&'static str> {
    let mut result = Vec::new();
    for part in value.split([';', ',', '|', ' ', '\t', '\r', '\n']) {
        let part = part.trim().trim_start_matches('*').to_ascii_lowercase();
        if part.is_empty() {
            continue;
        }
        let normalized = if part.starts_with('.') {
            part
        } else {
            format!(".{part}")
        };
        if let Some(extension) = TEXT_EXTENSIONS
            .iter()
            .copied()
            .find(|extension| *extension == normalized)
        {
            if !result.contains(&extension) {
                result.push(extension);
            }
        }
    }
    result
}

fn with_key<F>(path: &str, action: F) -> Result<(), String>
where
    F: FnOnce(HKEY) -> Result<(), String>,
{
    let path = wide(path);
    let mut key = HKEY::default();
    let status = unsafe { RegCreateKeyW(HKEY_CURRENT_USER, PCWSTR(path.as_ptr()), &mut key) };
    if status != ERROR_SUCCESS {
        return Err(format!(
            "无法创建注册表键 {}（错误 {}）",
            path_name(&path),
            status.0
        ));
    }
    let result = action(key);
    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

unsafe fn open_key(path: &str, access: REG_SAM_FLAGS) -> Option<HKEY> {
    let path = wide(path);
    let mut key = HKEY::default();
    let status = RegOpenKeyExW(
        HKEY_CURRENT_USER,
        PCWSTR(path.as_ptr()),
        Some(0),
        access,
        &mut key,
    );
    if status == ERROR_SUCCESS {
        Some(key)
    } else {
        None
    }
}

fn read_sz(path: &str, name: Option<&str>) -> Option<String> {
    unsafe {
        let key = open_key(path, KEY_READ)?;
        let name = name.map(wide);
        let name = name
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or(PCWSTR::null());
        let mut size = 0u32;
        let status = RegQueryValueExW(key, name, None, None, None, Some(&mut size));
        if status != ERROR_SUCCESS || size < 2 {
            let _ = RegCloseKey(key);
            return None;
        }
        let mut data = vec![0u16; ((size as usize) + 1) / 2];
        let mut actual = size;
        let status = RegQueryValueExW(
            key,
            name,
            None,
            None,
            Some(data.as_mut_ptr().cast::<u8>()),
            Some(&mut actual),
        );
        let _ = RegCloseKey(key);
        if status != ERROR_SUCCESS {
            return None;
        }
        let end = data
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(data.len());
        Some(String::from_utf16_lossy(&data[..end]))
    }
}

fn read_dword(path: &str, name: Option<&str>) -> Option<u32> {
    unsafe {
        let key = open_key(path, KEY_READ)?;
        let name = name.map(wide);
        let name = name
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or(PCWSTR::null());
        let mut value = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let status = RegQueryValueExW(
            key,
            name,
            None,
            None,
            Some((&mut value as *mut u32).cast::<u8>()),
            Some(&mut size),
        );
        let _ = RegCloseKey(key);
        if status == ERROR_SUCCESS && size == std::mem::size_of::<u32>() as u32 {
            Some(value)
        } else {
            None
        }
    }
}

fn delete_tree(path: &str) {
    let path = wide(path);
    unsafe {
        let _ = RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(path.as_ptr()));
    }
}

fn delete_value(path: &str, name: Option<&str>) {
    if let Some(key) = unsafe { open_key(path, KEY_SET_VALUE) } {
        let name = name.map(wide);
        let name = name
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or(PCWSTR::null());
        unsafe {
            let _ = RegDeleteValueW(key, name);
            let _ = RegCloseKey(key);
        }
    }
}

fn set_sz(key: HKEY, name: Option<&str>, value: &str) -> Result<(), String> {
    let name = name.map(wide);
    let name = name
        .as_ref()
        .map(|value| PCWSTR(value.as_ptr()))
        .unwrap_or(PCWSTR::null());
    let data: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), data.len() * 2) };
    let status = unsafe { RegSetValueExW(key, name, None, REG_SZ, Some(bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("无法写入注册表值（错误 {}）", status.0))
    }
}

fn set_none(key: HKEY, name: &str) -> Result<(), String> {
    let name = wide(name);
    let status = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_NONE, None) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("无法写入注册表值（错误 {}）", status.0))
    }
}

fn set_dword(key: HKEY, name: &str, value: u32) -> Result<(), String> {
    let name = wide(name);
    let bytes = value.to_le_bytes();
    let status =
        unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_DWORD, Some(&bytes)) };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("无法写入注册表值（错误 {}）", status.0))
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn path_name(path: &[u16]) -> String {
    let end = path
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(path.len());
    String::from_utf16_lossy(&path[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn text_extensions_are_unique_and_normalized() {
        let mut seen = HashSet::new();
        for extension in TEXT_EXTENSIONS {
            assert!(extension.starts_with('.'));
            assert_eq!(*extension, extension.to_ascii_lowercase());
            assert!(seen.insert(*extension));
        }
        assert!(seen.contains(".html"));
        assert!(seen.contains(".txt"));
        assert!(seen.contains(".rs"));
    }

    #[test]
    fn recommended_extensions_are_known() {
        let known: HashSet<&str> = TEXT_EXTENSIONS.iter().copied().collect();
        assert!(!RECOMMENDED_EXTENSIONS.is_empty());
        for extension in RECOMMENDED_EXTENSIONS {
            assert!(known.contains(extension));
        }
    }

    #[test]
    fn custom_extension_list_is_normalized() {
        let parsed = parse_extension_list("md; .TXT, unknown;*.Json");
        assert_eq!(parsed, vec![".md", ".txt", ".json"]);
    }

    #[test]
    fn association_modes_parse() {
        assert_eq!(AssociationMode::parse("all"), Some(AssociationMode::All));
        assert_eq!(
            AssociationMode::parse("default"),
            Some(AssociationMode::Recommended)
        );
        assert_eq!(
            AssociationMode::parse("custom"),
            Some(AssociationMode::Custom)
        );
        assert_eq!(AssociationMode::parse("none"), Some(AssociationMode::None));
    }
}
