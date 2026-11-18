# 第三方软件声明

本项目的原创代码采用 MIT 许可证。发布的可执行文件和安装包还包含或使用了下列第三方组件。许可证审计日期为 2026-11-18。

## 审计结论

- 未发现 GPL、AGPL、LGPL 或其他要求本项目开源同类代码的强 copyleft 依赖。
- Rust 依赖的许可证均为 MIT、Apache-2.0，或与其兼容的 Unicode-3.0。
- Inno Setup 明确允许使用其生成的安装程序分发商业或非商业软件。
- 项目自带源码、图标、示例 Markdown、CSS、JavaScript 均为本项目原创资源，未引用第三方素材。
- 审计通过，可以分发源代码、可执行文件和安装包。

## Rust 运行时与标准库

`native-reader.exe` 为 Rust 编译生成的 x64 原生程序，静态链接了 Rust 标准库的部分代码。

- 项目：The Rust Programming Language
- 版本：Rust 1.99.1，`x86_64-pc-windows-msvc`
- 许可证：MIT OR Apache-2.0
- 版权：The Rust Project Contributors
- 完整说明：[COPYRIGHT](https://github.com/rust-lang/rust/blob/1.99.1/COPYRIGHT)
- 本地文本：[licenses/Rust-COPYRIGHT.txt](licenses/Rust-COPYRIGHT.txt)、[licenses/Rust-MIT.txt](licenses/Rust-MIT.txt)、[licenses/Apache-2.0.txt](licenses/Apache-2.0.txt)

## Rust crates

| 组件 | 版本 | 许可证 | 用途 |
|---|---:|---|---|
| `windows` | 0.62.2 | MIT OR Apache-2.0 | Win32 API 绑定 |
| `windows-collections` | 0.3.2 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-core` | 0.62.2 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-future` | 0.3.2 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | `windows` 编译期宏 |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | `windows` 编译期宏 |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-numerics` | 0.3.1 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-result` | 0.4.1 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `windows-threading` | 0.2.1 | MIT OR Apache-2.0 | `windows` 传递依赖 |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | 编译期宏依赖 |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | 编译期宏依赖 |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | 编译期宏依赖 |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | 编译期宏依赖 |

版权分别归 Microsoft Corporation、David Tolnay 及其他各 crate 贡献者所有。许可全文：

- [licenses/MIT-windows-rs.txt](licenses/MIT-windows-rs.txt)
- [licenses/MIT-dtolnay.txt](licenses/MIT-dtolnay.txt)
- [licenses/Apache-2.0.txt](licenses/Apache-2.0.txt)
- [licenses/Unicode-3.0.txt](licenses/Unicode-3.0.txt)

## Inno Setup

Windows 安装包使用 Inno Setup 7 生成，生成物包含 Inno Setup 运行时组件。

- 项目：Inno Setup
- 版权：Copyright (C) 1997-2026 Jordan Russell；Portions Copyright (C) 2000-2026 Martijn Laan
- 许可证允许任何人将 Inno Setup 用于商业或非商业用途，并分发由其生成的安装程序
- 本项目未修改 Inno Setup 本体，也未将其声称为原创软件
- 许可全文：[licenses/Inno-Setup.txt](licenses/Inno-Setup.txt)

## Windows 系统组件

程序调用 Windows 自带的 RichEdit、GDI、DWM、Shell 和注册表 API。这些系统组件随 Windows 提供，本项目不单独再分发其二进制文件。

## 项目资源

- `assets/native-reader.svg`、`assets/native-reader.ico`、`assets/native-reader-256.png`：本项目原创图标
- `samples/`：本项目原创示例文本和网页
- `src/`、`installer/`、`tools/`：本项目原创源代码和构建脚本

本项目使用的界面字体为 Windows 系统自带的 Microsoft YaHei UI 与 Consolas，不随安装包分发字体文件。
