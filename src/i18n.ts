export type Lang = "zh" | "en";

export const LANG_KEY = "phylopanel.lang";

type Table = Record<string, string>;

const zh: Table = {
  "header.binary": "可执行文件",
  "header.binaryPlaceholder": "内置 gotree 的路径，也可以是任意其他 cobra 程序",
  "header.choose": "选择…",
  "header.builtin": "用内置 GoTree",
  "header.builtinTitle": "解出程序内置的 gotree，无需另外安装命令行工具",
  "header.refresh": "重新反射命令表",
  "header.presets": "预设工作流",
  "header.language": "语言",
  "header.theme": "主题",

  "panel.commands": "命令表",
  "panel.searchPlaceholder": "搜索命令或说明",
  "panel.input": "输入",
  "panel.inputFile": "树文件（传 --input）",
  "panel.inputPaste": "直接粘贴",
  "panel.pathPlaceholder": "F:\\trees\\my.tre（支持 .gz / http:// / itol://）",
  "panel.params": "参数",
  "panel.workflow": "工作流",
  "panel.run": "运行",
  "panel.clear": "清空",
  "panel.cmdline": "等价命令行",
  "panel.copy": "复制",
  "panel.result": "结果",
  "panel.saveOutput": "保存输出…",

  "theme.light": "浅色",
  "theme.dark": "深色",
  "theme.auto": "跟随系统",

  "status.starting": "正在启动…",
  "status.noBinary": "还没有选择可执行文件。",
  "status.reflecting": "正在反射命令表…",
  "status.counts": "{commands} 个可执行命令 / {groups} 个顶层命令族",
  "status.cached": "命令表来自缓存",
  "status.reflected": "命令表已重新反射",

  "dialog.chooseBinary": "选择 cobra 命令行程序",
  "dialog.executableFilter": "可执行文件",
  "dialog.pickSave": "选择 {flag} 输出路径",
  "dialog.pickOpen": "选择 {flag}",

  "tree.executable": "可执行",
  "tree.dualTitle": "这个命令既能自己运行，也有子命令",

  "steps.empty": "从左侧命令表添加命令，多个命令会用管道依次串起来。",
  "form.pickStep": "选中一个步骤后在这里填参数。",
  "form.paramsOf": "参数 · {command}",
  "form.own": "命令参数",
  "form.ownHint": "该命令自己的开关",
  "form.shared": "通用参数",
  "form.sharedHint": "从上游命令继承的开关，多数时候不用动",
  "form.positional": "位置参数：{label}",
  "form.positionalHint": "cobra 不会在 --help 里声明这类参数，只能靠工具包人工标注",
  "form.noFlags": "该命令没有可配置参数，直接透传输入树。",
  "form.enable": "启用",
  "form.defaultOption": "(默认)",
  "form.default": "默认 {value}",
  "form.browse": "浏览…",

  "run.empty": "还没有添加任何命令。",
  "run.busy": "执行中…",
  "run.done": "完成 · {steps} 步 · {ms} ms",
  "run.failed": "{failed}/{steps} 步失败 · {message}",

  "out.execFailed": "执行失败",
  "out.rawStderr": "原始 stderr",
  "out.imageAlt": "输出图片",
  "out.treeAlt": "树图",
  "out.saveTitle": "保存输出",
  "out.saved": "已保存到 {path}",
  "out.copyDone": "命令已复制到剪贴板。",
  "tpl.missing": "模板里的命令在当前版本中不存在：{name}",

  "E.bundledWindowsOnly": "内置 GoTree 仅在 Windows 版本可用，请手动选择可执行文件。",
  "E.toolDirCreate": "无法创建工具目录：{detail}",
  "E.toolWrite": "无法写入内置工具：{detail}",
  "E.toolPlace": "无法就位内置工具：{detail}",
  "E.bundledExtract": "内置工具解出失败：{detail}",
  "E.binaryMissing": "找不到可执行文件：{detail}",
  "E.reflect": "反射失败：{detail}",
  "E.execute": "执行失败：{detail}",
  "E.write": "写入失败：{detail}",
  "E.base64": "base64 解码失败：{detail}",
  "E.noSteps": "没有可执行的步骤",
  "E.noUpstream": "上游进程没有可衔接的输出",
  "E.spawn": "无法启动：{detail}",
  "E.wait": "等待子进程失败：{detail}",
};

const en: Table = {
  "header.binary": "Executable",
  "header.binaryPlaceholder": "Path to the bundled gotree, or to any other cobra CLI",
  "header.choose": "Browse…",
  "header.builtin": "Use bundled GoTree",
  "header.builtinTitle": "Extracts the gotree binary shipped inside PhyloPanel, so nothing else has to be installed",
  "header.refresh": "Re-reflect commands",
  "header.presets": "Preset workflows",
  "header.language": "Language",
  "header.theme": "Theme",

  "panel.commands": "Commands",
  "panel.searchPlaceholder": "Search commands or descriptions",
  "panel.input": "Input",
  "panel.inputFile": "Tree file (passes --input)",
  "panel.inputPaste": "Paste directly",
  "panel.pathPlaceholder": "F:\\trees\\my.tre (.gz / http:// / itol:// supported)",
  "panel.params": "Parameters",
  "panel.workflow": "Workflow",
  "panel.run": "Run",
  "panel.clear": "Clear",
  "panel.cmdline": "Equivalent command line",
  "panel.copy": "Copy",
  "panel.result": "Result",
  "panel.saveOutput": "Save output…",

  "theme.light": "Light",
  "theme.dark": "Dark",
  "theme.auto": "System",

  "status.starting": "Starting…",
  "status.noBinary": "No executable selected yet.",
  "status.reflecting": "Reflecting the command tree…",
  "status.counts": "{commands} runnable commands / {groups} top-level groups",
  "status.cached": "command tree from cache",
  "status.reflected": "command tree re-reflected",

  "dialog.chooseBinary": "Select a cobra command-line program",
  "dialog.executableFilter": "Executable",
  "dialog.pickSave": "Choose the output path for {flag}",
  "dialog.pickOpen": "Choose a file for {flag}",

  "tree.executable": "runnable",
  "tree.dualTitle": "This command runs on its own and also has subcommands",

  "steps.empty": "Add commands from the list on the left. Several commands are chained with pipes.",
  "form.pickStep": "Select a step to edit its parameters here.",
  "form.paramsOf": "Parameters · {command}",
  "form.own": "Command flags",
  "form.ownHint": "Flags that belong to this command only",
  "form.shared": "Shared flags",
  "form.sharedHint": "Flags inherited from the root command; usually best left alone",
  "form.positional": "Positional arguments: {label}",
  "form.positionalHint": "Cobra never advertises these in --help, so they are curated in the tool pack",
  "form.noFlags": "This command takes no flags; the input tree passes straight through.",
  "form.enable": "Enable",
  "form.defaultOption": "(default)",
  "form.default": "default {value}",
  "form.browse": "Browse…",

  "run.empty": "No commands have been added yet.",
  "run.busy": "Running…",
  "run.done": "Done · {steps} steps · {ms} ms",
  "run.failed": "{failed} of {steps} steps failed · {message}",

  "out.execFailed": "Failed",
  "out.rawStderr": "Raw stderr",
  "out.imageAlt": "Output image",
  "out.treeAlt": "Tree drawing",
  "out.saveTitle": "Save output",
  "out.saved": "Saved to {path}",
  "out.copyDone": "Command copied to the clipboard.",
  "tpl.missing": "This template uses commands that are absent from the current version: {name}",

  "E.bundledWindowsOnly": "The bundled GoTree is only available in the Windows build. Pick an executable yourself.",
  "E.toolDirCreate": "Could not create the tools directory: {detail}",
  "E.toolWrite": "Could not write the bundled tool: {detail}",
  "E.toolPlace": "Could not move the bundled tool into place: {detail}",
  "E.bundledExtract": "Could not extract the bundled tool: {detail}",
  "E.binaryMissing": "Executable not found: {detail}",
  "E.reflect": "Reflecting the command tree failed: {detail}",
  "E.execute": "Execution failed: {detail}",
  "E.write": "Could not write the file: {detail}",
  "E.base64": "base64 decoding failed: {detail}",
  "E.noSteps": "There are no steps to run",
  "E.noUpstream": "The previous step produced no output to pipe",
  "E.spawn": "Could not start: {detail}",
  "E.wait": "Waiting for the child process failed: {detail}",
};

const TABLES: Record<Lang, Table> = { zh, en };

const HTML_LANG: Record<Lang, string> = { zh: "zh-CN", en: "en" };

export function detect(): Lang {
  const stored = localStorage.getItem(LANG_KEY);
  if (stored === "zh" || stored === "en") return stored;
  const browser = (navigator.language ?? "").toLowerCase();
  return browser.startsWith("zh") ? "zh" : "en";
}

let current: Lang = detect();

export function lang(): Lang {
  return current;
}

export function setLang(value: Lang) {
  current = value;
  localStorage.setItem(LANG_KEY, value);
}

/** Missing English entries fall back to the Chinese source text, then to the key itself. */
export function t(key: string, params?: Record<string, string | number>): string {
  const raw = TABLES[current][key] ?? TABLES.zh[key] ?? key;
  if (!params) return raw;
  return raw.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match
  );
}

/**
 * Rust hands back `E#key|detail` instead of prose so the wording can follow the UI
 * language. Anything that does not match is a plain message and stays untouched.
 */
export function describeError(raw: string): string {
  if (!raw.startsWith("E#")) return raw;
  const body = raw.slice(2);
  const split = body.indexOf("|");
  const key = split < 0 ? body : body.slice(0, split);
  const detail = split < 0 ? "" : body.slice(split + 1);
  const known = TABLES.zh[`E.${key}`] !== undefined;
  if (!known) return raw;
  return t(`E.${key}`, { detail });
}

const ATTRIBUTES: [string, string][] = [
  ["placeholder", "i18nPlaceholder"],
  ["title", "i18nTitle"],
  ["aria-label", "i18nAriaLabel"],
];

export function localizeStatic(root: ParentNode = document) {
  root.querySelectorAll<HTMLElement>("[data-i18n]").forEach((node) => {
    node.textContent = t(node.dataset.i18n!);
  });
  for (const [attribute, dataKey] of ATTRIBUTES) {
    for (const node of root.querySelectorAll<HTMLElement>(`[${attribute}]`)) {
      const key = node.dataset[dataKey];
      if (key) node.setAttribute(attribute, t(key));
    }
  }
}

export function applyLang(value: Lang) {
  setLang(value);
  document.documentElement.lang = HTML_LANG[value];
  document.documentElement.dataset.lang = value;
  localizeStatic();
}

/** Tool pack entries are authored in English with an optional Chinese sibling. */
export function localize(zh: string | null | undefined, english: string): string {
  return current === "zh" && zh ? zh : english;
}

/** Keys present in one language but missing in the other; used by the UI checks. */
export function missingKeys(): string[] {
  const zhKeys = Object.keys(zh);
  const enKeys = Object.keys(en);
  const onlyZh = zhKeys.filter((k) => !enKeys.includes(k));
  const onlyEn = enKeys.filter((k) => !zhKeys.includes(k));
  return [...onlyZh, ...onlyEn].map((k) => (onlyZh.includes(k) ? `zh-only:${k}` : `en-only:${k}`));
}
