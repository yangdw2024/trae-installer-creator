use serde::{Deserialize, Serialize};

pub fn default_license_text() -> String {
    concat!(
        "最终用户许可协议\n",
        "==========================\n\n",
        "重要 - 请仔细阅读：\n\n",
        "本最终用户许可协议（以下简称\"协议\"）是您（个人或单一实体）与软件作者之间就上述软件产品达成的法律协议。\n\n",
        "安装、复制或以其他方式使用本软件，即表示您同意受本协议条款的约束。如果您不同意本协议的条款，请不要安装或使用本软件。\n\n",
        "1. 许可授予\n",
        "   您可以在单台计算机上安装和使用本软件的一份副本。\n\n",
        "2. 其他权利和限制说明\n",
        "   - 不得对本软件进行反向工程、反编译或反汇编。\n",
        "   - 本软件按\"现状\"提供，不提供任何形式的保证。\n\n",
        "3. 版权\n",
        "   本软件的所有所有权和版权均归软件作者所有。\n\n",
        "4. 无担保\n",
        "   本软件按\"现状\"提供，不提供任何明示或暗示的担保。\n\n",
        "5. 责任限制\n",
        "   在任何情况下，作者均不对因使用本软件而产生的任何损害承担责任。\n"
    )
    .to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SetupSection {
    pub app_name: String,
    pub app_version: String,
    pub app_publisher: String,
    pub app_publisher_url: String,
    pub default_dir_name: String,
    pub default_group_name: String,
    pub uninstall_display_icon: String,
    pub compression: String,
    pub output_dir: String,
    pub output_base_filename: String,
    pub license_text: String,
    pub privileges_required: String,
    pub wizard_style: String,
    pub solid_compression: bool,
}

impl Default for SetupSection {
    fn default() -> Self {
        Self {
            app_name: "我的程序".to_string(),
            app_version: "1.0".to_string(),
            app_publisher: "我的公司".to_string(),
            app_publisher_url: String::new(),
            default_dir_name: r"{autopf}\我的程序".to_string(),
            default_group_name: "我的程序".to_string(),
            uninstall_display_icon: r"{app}\MyProg.exe".to_string(),
            compression: "zlib".to_string(),
            output_dir: ".".to_string(),
            output_base_filename: String::new(),
            license_text: default_license_text(),
            privileges_required: "lowest".to_string(),
            wizard_style: "modern".to_string(),
            solid_compression: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct FileEntry {
    pub source: String,
    pub dest_dir: String,
    pub dest_name: String,
    pub flags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct IconEntry {
    pub name: String,
    pub filename: String,
    pub parameters: String,
    pub working_dir: String,
    pub icon_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RegistryEntry {
    pub root: String,
    pub subkey: String,
    pub value_type: String,
    pub value_name: String,
    pub value_data: String,
    pub flags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct RunEntry {
    pub filename: String,
    pub parameters: String,
    pub working_dir: String,
    pub description: String,
    pub flags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default)]
pub struct SetupScript {
    pub setup: SetupSection,
    pub files: Vec<FileEntry>,
    pub icons: Vec<IconEntry>,
    pub registry: Vec<RegistryEntry>,
    pub run: Vec<RunEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArchiveFileMeta {
    pub rel_path: String,
    pub size: u64,
    pub crc32: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallerPayloadConfig {
    pub setup: SetupSection,
    pub icons: Vec<IconEntry>,
    pub registry: Vec<RegistryEntry>,
    pub run: Vec<RunEntry>,
    pub files_meta: Vec<ArchiveFileMeta>,
    pub total_uncompressed_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UninstallManifest {
    pub app_name: String,
    pub app_version: String,
    pub publisher: String,
    pub install_dir: String,
    pub installed_files: Vec<String>,
    pub shortcuts: Vec<String>,
    pub registry_subkey: String,
}
