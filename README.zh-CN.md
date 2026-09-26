# PhyloPanel

基于 [cobra](https://github.com/spf13/cobra) 构建的系统发育学命令行工具 Windows 图形操作面板。
**内置 GoTree v0.5.2**，装好即用；反射引擎本身是通用的，把面板指向其他任意 cobra 程序（例如
GoAlign）就会变成那个程序的面板。

它会反射你所指定的可执行文件的完整命令树，把每个命令变成带类型校验的表单，用真实的
操作系统管道把多个命令串起来，并显示它实际执行的等价命令行。GUI 从不隐藏任何东西：
你在面板里搭出的每一步都可以原样复制到终端里运行。

[English](README.md) · **简体中文** · [详细使用文档](docs/zh/README.md)

---

## 为什么做这个

系统发育学命令行工具功能很强，但手工驱动的成本很高。仅 GoTree 一个工具就有分布在 35 个
顶层命令组里的 82 个可执行命令，GoAlign 还有几十个。想把它们串起来，就得记住每个命令
用哪个参数接收输入、哪个子命令会"吃掉"树而不往下传、以及带空格的路径该怎么加引号。

PhyloPanel 始终把命令行工具当作唯一的事实来源——它一个命令都不重新实现——只把打字的负担
拿掉：

- **没有安装步骤。** GoTree 已经编译进主程序，下载到的就是一个 `.exe`，首次启动自动解出。
- **界面是生成的，不是手写的。** 参数、类型、默认值全部来自 `<tool> --help`，命令行工具升级
  后面板不需要跟着改代码。
- **等价命令行始终可见。** 你可以用面板反过来学命令行，学会之后随时脱离面板。

## 功能

| | |
|---|---|
| 界面语言 | 中文 / English，运行时即时切换，跨启动记忆 |
| 配色主题 | 浅色 / 深色 / 跟随系统，连 Windows 标题栏一起变 |
| 命令覆盖 | 目标命令行的每一个可执行命令，按命令组折叠成树 |
| 搜索 | 按命令名或说明过滤命令树 |
| 管道工作流 | 按顺序叠加命令，第 *n* 步的 stdout 直接接进第 *n+1* 步的 stdin |
| 预设工作流 | 每个工具自带若干一键可用的多步流程 |
| 参数表单 | 类型化输入框、路径类参数带文件选择器、枚举参数带下拉框 |
| 结果展示 | 树、统计表、SVG 与 PNG 图形直接在界面内渲染 |
| 导出 | 输出可另存为 `.tree` / `.svg` / `.png` / `.txt`，或复制等价命令行 |
| 换工具也能用 | 指向任意 cobra 程序，它就变成那个程序的面板（仅反射；目前只有 GoTree 有人工整理的预设） |

## 到底打包了什么

| 工具 | 是否内置在 exe 里 | 预设 + 位置参数提示 |
|---|---|---|
| **GoTree v0.5.2** | 是，首次启动自动解出 | 有，整理在 `src-tauri/toolpacks/gotree.json` |
| **GoAlign** | 否，需自备 `goalign.exe` | **暂无** —— 还没有 GoAlign 工具包 |
| 其他任意 cobra 程序 | 否 | 除非你自己加工具包 |

反射本身是通用的：不论哪个工具，命令、参数、类型、默认值都会照常列出来。*工具包*额外补的是
`--help` 表达不了的那一层——预设工作流，以及某些命令必需的裸位置参数。所以现在驱动 GoAlign，
每一步都得手工搭。详见[命令覆盖](docs/zh/command-reference.md)。

## 快速开始

1. 从 [Releases](https://github.com/ZengZichao/PhyloPanel-Windows/releases/latest) 下载
   **PhyloPanel-0.1.0-win-x64.exe** —— 单个自包含文件，没有安装程序。
2. 双击运行。状态栏应当显示
   `GoTree  ·  v0.5.2  ·  82 个可执行命令 / 35 个顶层命令族`。
3. 在**输入**区里，要么直接粘贴 Newick 字符串，要么输入/选择树文件路径。
4. 点击左侧**命令表**里的某个命令——它会出现在**工作流**中。
5. 按**运行**。树图或表格出现在**结果**区，**等价命令行**显示刚才真正执行的内容。

想立刻体验多步流程，直接点顶栏的预设工作流按钮。

**环境要求：** Windows 10 或 11，以及 [WebView2 Runtime](https://learn.microsoft.com/zh-cn/microsoft-edge/webview2/)
（Windows 11 自带；较老的 Windows 10 可能需要自行安装）。不需要 Python、.NET、Go、Node.js。

## 语言与主题

两个开关都在顶栏右侧。

- **语言**在中文和英文之间即时切换整个界面，并记住选择；没有手动选过时，跟随系统语言。
- **主题**提供*跟随系统*、*浅色*、*深色*三档。选择会同时改变 Windows 原生标题栏，
  且"跟随系统"会实时响应系统设置的改动。

命令名和参数说明保持英文，这是有意为之：它们来自上游工具自己的 `--help` 输出，而不是
PhyloPanel 的文案。预设工作流名称和位置参数提示有中文，因为这两类内容是 PhyloPanel 自己
整理的。详见[语言与主题](docs/zh/language-and-theme.md)。

## 工作原理

PhyloPanel 是一个 Tauri v2 应用：Rust 后端负责进程，WebView2 窗口负责渲染 TypeScript 前端。

它从不链接或引入上游代码。每一步都是一个独立的子进程，进程之间只通过 stdout → stdin 传递，
和你自己在终端里敲 `gotree reroot midpoint | gotree draw svg` 完全等价。子进程以
`CREATE_NO_WINDOW` 启动，所以不会闪出黑色控制台窗口。

只是重复命令行默认值的参数会在启动进程之前被剔除，这样等价命令行既短，结果又和手敲一致。

```
  ┌──────────────┐   启动    ┌──────────┐   管道   ┌──────────
  │ PhyloPanel   │ ────────► │ gotree   │ ───────► │ gotree   │
  │  （Rust+UI） │  只传 argv │ reroot   │  stdout  │  draw    │
  └──────────────┘           └──────────┘          └──────────
```

## 文档

| 页面 | 内容 |
|---|---|
| [快速上手](docs/zh/getting-started.md) | 首次启动、两种输入方式、一个完整例子 |
| [界面详解](docs/zh/interface.md) | 逐个区域、控件与状态提示的说明 |
| [工作流与管道](docs/zh/workflows.md) | 多步串联、预设、参数规则、位置参数 |
| [语言与主题](docs/zh/language-and-theme.md) | 哪些内容会被翻译、哪些不会、选择存在哪里 |
| [命令覆盖](docs/zh/command-reference.md) | 命令表如何反射、GoTree 命令组、如何为其他 CLI 写工具包 |
| [从源码构建](docs/zh/building-from-source.md) | 工具链、开发调试、发布构建、跑测试 |
| [故障排查](docs/zh/troubleshooting.md) | 常见失败与如何读懂它们 |

英文文档在 [`docs/en/`](docs/en/README.md)。

## 界面预览

| 英文 / 浅色 | 中文 / 深色 |
|---|---|
| ![英文浅色界面](docs/images/interface-en-light.png) | ![中文深色界面](docs/images/interface-zh-dark.png) |

## 从源码构建

```bash
npm install
npm run build
npx tauri build --no-bundle
# → src-tauri/target/release/PhyloPanel.exe
```

需要带 MSVC 后端的 Rust 工具链、Visual Studio 2022 Build Tools、Node 18+，以及 WebView2
Runtime。完整流程（包括内置的 GoTree 二进制是怎么编出来的）见[从源码构建](docs/zh/building-from-source.md)。

## 目录结构

```
src/                 TypeScript 前端
  i18n.ts            中英双语文案表、查找与 DOM 本地化
  theme.ts           浅色 / 深色 / 跟随系统的解析
  main.ts            面板状态、渲染、IPC 调用
  controls.ts        单命令参数表单的构建
  types.ts           共享类型与命令树构建
index.html           静态骨架，用 data-i18n 标注
src-tauri/
  src/               Rust：命令反射、管道执行、内置解出、IPC 命令
  toolpacks/         按工具整理的人工标注（显示名、预设、位置参数提示）
  tools/gotree.exe   内置命令行工具，首次启动时解出
docs/en, docs/zh     详细使用文档，一棵语言树一种语言
samples/demo.tre     六棵分类单元的 Newick 树，用来试功能
```

## 许可证

PhyloPanel 自身源码以 [Apache License 2.0](LICENSE) 发布，另见 [NOTICE](NOTICE)。

```
Copyright 2026 ZengZichao

本程序依据 Apache License 2.0 发布，全文见 LICENSE 文件。
```

PhyloPanel 是一个独立的程序，它以子进程方式启动 [GoTree](https://github.com/choishingwan/GoTree)
并通过管道与它通信——对你自己提供的其他 cobra 程序（例如
[GoAlign](https://github.com/choishingwan/GoAlign)）也同样如此。它不链接、也不引入两者的任何
代码，因此双方始终是相互独立、各自持证的作品。两个上游工具均为 GNU GPL v2，其许可证文本与
著作权仍归其作者，GPL-2.0 全文已原样收录在
[LICENSES/GPL-2.0.txt](LICENSES/GPL-2.0.txt)。本仓库与发布的 exe 中都不含 GoAlign 的任何副本。

内置的 `src-tauri/tools/gotree.exe` 是 GoTree v0.5.2 的未修改构建产物——只按上游 Makefile 本来
的做法注入了版本字符串。它对应的源码就是上述上游仓库的该 tag，本项目不对其附加任何超出 GPL v2
的限制。

## 致谢

本面板所驱动的这些命令行工具，作者是 [Hing Wan Chang](https://github.com/choishingwan)。
