# 从源码构建

## 工具链

| 组件 | 版本 | 说明 |
|---|---|---|
| Rust | stable，MSVC host（`x86_64-pc-windows-msvc`） | `rustup show` 应能看到 MSVC 工具链 |
| C++ 构建工具 | Visual Studio 2022 Build Tools | 需要"使用 C++ 的桌面开发"负载；Rust 链接要用它的 `link.exe` |
| Node.js | 18 或更高 | 实测 24 |
| WebView2 Runtime | Evergreen | Windows 11 默认自带 |
| Git | 任意 | 仅用于克隆 |

```bash
git clone https://github.com/ZengZichao/PhyloPanel-Windows.git
cd PhyloPanel-Windows
npm install
```

## 开发调试

涉及两个进程：Vite 提供前端，Rust 宿主窗口加载它。

```bash
# 终端 1
npm run dev            # vite 监听 http://localhost:1420

# 终端 2
cd src-tauri
cargo run              # debug 宿主，读取 tauri.conf.json 里的 devUrl
```

debug 构建指向开发地址，所以改前端会热更新；改 Rust 需要重新 `cargo run`。
也可以用 `npx tauri dev` 一次拉起两边。

## 测试

```bash
cd src-tauri
cargo test --release
```

共 8 个测试：5 个纯单元测试，覆盖 `--help` 解析、参数物化和 shell 引号；另外 3 个端到端测试会
驱动**真实的** GoTree 二进制跑完整管道。不指定二进制时，端到端测试自动跳过：

```bash
GOTREE_BIN=/path/to/gotree.exe cargo test --release
```

用 `src-tauri/tools/gotree.exe` 即可，所以刚克隆下来就能跑全套。

前端没有测试运行器；`npm run build` 会先跑 `tsc --noEmit`，那就是它的编译期闸门。

## 发布构建

```bash
npm run build                 # tsc --noEmit，然后 vite build → dist/
npx tauri build --no-bundle   # → src-tauri/target/release/phylopanel.exe
```

`--no-bundle` 跳过 NSIS 安装包，只留下约 29 MB 的单文件自包含 exe——这个软件就是按"一个文件"
来设计和分发的。去掉该参数则会在 `src-tauri/target/release/bundle/` 下得到安装包。

### exe 里到底装了什么

- Rust 主程序，内嵌从 `dist/` 编译进来的前端资源。
- `src-tauri/tools/gotree.exe`，通过 `src/bundle.rs` 里的 `include_bytes!` 编进主程序，首次启动
  解到 `%APPDATA%\app.phylo.panel\tools\`，文件名取内容 FNV-1a 指纹。写入时先落 `.part` 再改名，
  所以中途崩溃绝不会留下一个看起来合法的半成品工具。

正因为工具是内嵌的，`src-tauri/tools/gotree.exe` **必须提交进仓库**，尽管其他 `.exe` 都被忽略——
见 `.gitignore` 里 `!src-tauri/tools/*.exe` 这条例外。

### 重新构建内置的 GoTree

只在需要跟进上游新版本时才要做：

```bash
git clone https://github.com/choishingwan/GoTree
cd GoTree
go build -trimpath -ldflags="-s -w -X github.com/evolbioinfo/gotree/cmd.Version=v0.5.2" -o ../PhyloPanel-Windows/src-tauri/tools/gotree.exe .
```

`-X ...cmd.Version` 这个注入不能省：不写它，`gotree version` 就什么都不输出，面板状态栏里的版本
字段会是空的。上游自己的 Makefile 也是这样注入的。

国内网络可能需要 `GOPROXY=https://goproxy.cn,direct`。产物放进去后，删掉反射缓存再重建面板。

## 新增语言或主题

- **语言：** 在 `src/i18n.ts` 加一张文案表，扩展 `Lang` 联合类型，再给工具包加对应的 `*Zh` 类
  字段。`missingKeys()` 会列出只在一种语言里定义的键。
- **主题：** 在 `src/styles.css` 加一个 `[data-theme="…"]` 块，覆盖同一批自定义属性；然后扩展
  `src/theme.ts` 里的 `ThemeChoice` 和 `index.html` 里 `<select id="theme">` 的选项。没有规则
  硬编码颜色，所以新配色只是改 token。

## 目录结构

```
src/                 前端
  i18n.ts            中英双语表、t()、DOM 本地化、后端错误码解析
  theme.ts           主题解析、持久化、标题栏同步
  main.ts            状态、渲染、IPC 调用
  controls.ts        参数表单构建
  types.ts           共享类型、命令树构建、参数物化
index.html           带 data-i18n 标注的静态骨架
src-tauri/
  src/lib.rs         模块装配与 E#<key>|<detail> 错误码辅助函数
  src/schema.rs      把 --help 解析成带类型的命令 schema
  src/runner.rs      进程管道、argv 构建、shell 引号
  src/commands.rs    六个 IPC 命令
  src/bundle.rs      命令行工具的内嵌与解出
  src/toolpack.rs    工具包加载与匹配
  toolpacks/         按工具整理的人工标注 JSON
  capabilities/      Tauri 权限集（注意 core:window:allow-set-theme）
  tauri.conf.json    窗口、CSP、构建钩子
docs/en, docs/zh     本文档
samples/demo.tre     示例输入
```

## 构建问题排查

**改过项目目录名之后构建失败。** Cargo 在 `src-tauri/target` 里缓存了绝对路径，删掉该目录重建。

**找不到 `link.exe`，或报 MSVC 工具链相关的 panic。** 安装或修复 Visual Studio 2022 Build Tools
的 C++ 负载，并确认 Rust host 三元组是 `x86_64-pc-windows-msvc`。

**窗口开出来是空白。** Vite 没起来，或者不在 1420 端口。先跑 `npm run dev`；或者改用发布构建，
它读内嵌资源而不是开发地址。

**程序能开但命令表是空的。** 内置工具没解出来——检查 `%APPDATA%\app.phylo.panel\tools\` 和状态栏
提示的原文。
