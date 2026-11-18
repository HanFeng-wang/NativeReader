# Native Reader

<img src="assets/native-reader-256.png" width="96" alt="Native Reader 图标">

一个用 Rust、Win32 和 RichEdit 编写的极简 Windows 文本查看器。没有 Electron，没有 WebView，也没有解释器运行时。启动快、常驻内存低，面向 Markdown、源码和任意纯文本文件。

## 特点

- **原生实现**：编译为单个 x64 可执行文件，界面直接使用 Windows 原生控件。
- **适合大文件**：不启动浏览器内核；单文件读取上限 128 MB，500 KB 以上的 Markdown 或源码仍可流畅滚动和跳转。
- **Markdown 预览与编辑**：支持标题、列表、引用、任务列表、表格、链接、行内代码和围栏代码块。
- **AI 风格代码块**：独立灰底区域、左上角语言标签、右上角复制按钮。
- **标题目录**：Markdown 预览工具栏提供“目录”，可快速跳转到任意标题。
- **源码查看器**：行号、语法着色、代码块复制、关闭自动换行，接近 VS Code 的只读/编辑体验。
- **查找替换**：`Ctrl+F` 查找、`F3` / `Shift+F3` 前后跳转，支持区分大小写、全字匹配、替换和全部替换。
- **右键菜单**：复制、剪切、粘贴纯文本、撤销、全选、复制全文。
- **状态栏**：显示行列号、文件类型、编码、文件大小、查看模式和修改状态。
- **HTML 工作流**：在程序内编辑 HTML 源码；点击“浏览器预览”交给系统默认浏览器打开，相对 CSS 和 JavaScript 会正常加载。
- **多窗口**：可以同时打开多个窗口；同一路径只会激活已有窗口，避免重复编辑同一文件。
- **其他**：浅色/深色主题、窗口置顶、字号缩放、拖拽打开、外部修改后自动重新加载。

## 技术栈

| 层 | 技术 |
|---|---|
| 语言 | Rust 2021 |
| UI | Win32 + RichEdit |
| 渲染 | 自研 Markdown → RTF、语法高亮 → RichEdit |
| 安装器 | Inno Setup 7 |
| 运行时 | 无额外运行时，单个原生 EXE |

## 功能支持

| 格式 | 查看 | 编辑 | 预览 |
|---|---:|---:|---:|
| Markdown | 是 | 是 | 原生只读预览 |
| HTML / XHTML | 是 | 是 | 调用默认浏览器 |
| 代码 / 配置 / 纯文本 | 是 | 是 | 语法着色源码视图 |

已内置着色的语言包括 Rust、C、C++、C#、Java、JavaScript、TypeScript、Python、Go、JSON、TOML、YAML、SQL、Shell、PowerShell、HTML、CSS、Markdown、INI、Lua 和批处理。其他文本按纯文本显示。

文本编码自动识别 UTF-8、UTF-8 BOM、UTF-16 LE/BE 和常见 ANSI/GBK。保存时，已识别编码会尽量保持。

## 安装

从 [Releases](http://localhost:3000/han/NativeReader/releases) 下载 `NativeReader-Setup-0.1.0.exe`，双击运行即可。

- 按当前用户安装到 `%LOCALAPPDATA%\Programs\Native Reader`，无需管理员权限。
- 安装向导可选择“全查看”“默认查看”“用户自定义查看”或“仅安装，不修改文件关联”。
- Windows 10/11 可能保护已有的默认程序设置；安装器会注册“打开方式”，必要时引导你进入系统“默认应用”确认。
- 安装包尚未购买代码签名证书，首次运行时可能出现 SmartScreen 提示。请对照 Release 中的 `SHA256SUMS.txt` 校验文件。

## 快捷键

| 快捷键 | 功能 |
|---|---|
| `Ctrl+O` | 打开文件 |
| `Ctrl+S` / `Ctrl+Shift+S` | 保存 / 另存为 |
| `Ctrl+Shift+V` | 将剪贴板内容作为临时文档打开 |
| `Ctrl+Shift+C` | 复制当前文件全文 |
| `Ctrl+F` | 查找 |
| `F3` / `Shift+F3` | 下一个 / 上一个匹配项 |
| `Esc` | 关闭查找栏 |
| `Ctrl+1` / `Ctrl+2` | Markdown 预览 / 源码 |
| `Ctrl+T` | 切换窗口置顶 |
| `Ctrl+W` | 切换代码自动换行 |
| `Ctrl+Shift+D` | 切换浅色/深色主题 |
| `Ctrl++` / `Ctrl+-` / `Ctrl+0` | 放大 / 缩小 / 恢复字号 |
| `F5` | 重新载入文件 |

## 从源码构建

### 环境要求

- Windows 10/11 x64
- Rust stable，MSVC 工具链
- Visual Studio Build Tools，安装“使用 C++ 的桌面开发”（提供 `rc.exe`）
- [Inno Setup 7](https://jrsoftware.org/isinfo.php)（仅生成安装包时需要）

### 编译与测试

```bash
cargo test
cargo build --release
```

生成文件：`target\release\native-reader.exe`。

### 生成安装包

```powershell
& 'C:\Program Files\Inno Setup 7\ISCC.exe' installer\native-reader.iss
```

输出：`dist\NativeReader-Setup-0.1.0.exe`。

## 目录结构

```text
NativeReader/
├── assets/             # SVG 图标、ICO 与 Windows 资源
├── installer/          # Inno Setup 安装脚本与文件关联选项
├── licenses/           # 第三方许可证全文
├── samples/            # Markdown、HTML、CSS、JavaScript 示例
├── src/                # Rust 源码
├── tools/              # 图标生成脚本
├── build.rs            # 嵌入 Windows 图标和版本信息
├── Cargo.toml
└── Cargo.lock
```

## 说明

- Markdown 预览为只读；需要修改时按 `Ctrl+2` 切到源码。替换功能只在源码模式可用。
- HTML 不在程序内嵌入浏览器，也不会执行页面脚本；点击“浏览器预览”后由默认浏览器加载 HTML 及其相对 CSS/JS。
- 同一路径已打开时再次打开该文件，会激活现有窗口，而不是创建第二个编辑器实例。

## 开源与致谢

项目原创代码采用 [MIT License](LICENSE)。

使用的 Rust 标准库、`windows-rs`、`proc-macro2`、`quote`、`syn`、`unicode-ident` 和 Inno Setup 的许可证及版权信息见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。许可证审计确认不存在 GPL/AGPL 冲突，相关全文归档在 [licenses/](licenses/)。

---

喜欢的话，欢迎点一个 **Star**。
