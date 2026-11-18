# Native Reader 示例

这是一个使用 **Rust + Win32 + RichEdit** 编写的原生查看器。没有 Electron，没有 WebView，也没有解释器运行时。

## 文本与表格

| 功能 | 状态 | 说明 |
|---|---|---|
| Markdown 预览 | 支持 | 标题、列表、引用、表格、代码块 |
| 源码编辑 | 支持 | 语法着色、行号、查找替换 |
| HTML | 源码编辑 | “浏览器预览”调用系统默认浏览器 |
| 大文件 | 支持 | 按需渲染，避免浏览器内存开销 |

> 预览模式下不能编辑；按 `Ctrl+2` 切换到源码，按 `Ctrl+1` 回到预览。

## 任务列表

- [x] 原生窗口与置顶
- [x] 快捷复制、粘贴
- [x] 查找与替换
- [x] 行列号与编码状态栏
- [ ] 插件系统（不需要）

## 代码块

```rust
fn main() {
    let message = "原生、轻量、快速";
    println!("{message}");
}
```

```json
{
  "name": "NativeReader",
  "renderer": "RichEdit",
  "runtime": "native"
}
```

## 链接与中文

点击链接会交给系统浏览器，例如 [Rust 官网](https://www.rust-lang.org/)。中文、Unicode 和常见 ANSI/GBK 文本都可以自动识别。

---

用 `Ctrl+F` 查找，`F3` 查找下一个，`Shift+F3` 查找上一个。
