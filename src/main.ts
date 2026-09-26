import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { commandForm, type Values } from "./controls";
import {
  applyLang,
  describeError,
  lang,
  localize,
  t,
  type Lang,
} from "./i18n";
import { applyTheme, themeChoice, type ThemeChoice } from "./theme";
import {
  buildTree,
  materialFlags,
  type CommandSpec,
  type Reflected,
  type RunResult,
  type StepSpec,
  type ToolPack,
  type ToolSchema,
  type TreeNode,
} from "./types";

interface Draft {
  command: CommandSpec;
  values: Values;
  args: string[];
}

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

const state = {
  binary: localStorage.getItem("phylopanel.binary") ?? "",
  schema: null as ToolSchema | null,
  pack: null as ToolPack | null,
  steps: [] as Draft[],
  editing: -1,
  query: "",
  inputMode: "file" as "file" | "text",
  inputPath: localStorage.getItem("phylopanel.inputPath") ?? "",
  inputText: "",
  last: null as RunResult | null,
  busy: false,
};

let status: { render: () => string; kind: "" | "warn" | "error" } = {
  render: () => t("status.starting"),
  kind: "",
};

function setStatus(render: () => string, kind: "" | "warn" | "error" = "") {
  status = { render, kind };
  paintStatus();
}

function paintStatus() {
  const bar = $("statusline");
  bar.textContent = status.render();
  bar.className = `status ${status.kind}`;
}

function fail(message: string, kind: "" | "warn" | "error" = "error") {
  setStatus(() => describeError(message), kind);
}

async function reflect(force = false) {
  if (!state.binary) {
    setStatus(() => t("status.noBinary"), "warn");
    return;
  }
  setStatus(() => t("status.reflecting"));
  try {
    const result = await invoke<Reflected>("reflect", { binary: state.binary, force });
    state.schema = result.schema;
    state.pack = result.pack;
    const families = new Set(result.schema.commands.map((c) => c.group)).size;
    const commands = result.schema.commands.length;
    const version = result.schema.version;
    const fromCache = result.fromCache;
    const packName = result.pack.displayName;
    setStatus(
      () =>
        [
          packName || t("panel.commands"),
          version,
          t("status.counts", { commands, groups: families }),
          fromCache ? t("status.cached") : t("status.reflected"),
        ]
          .filter(Boolean)
          .join("  ·  ")
    );
    renderTree(state.query);
    renderTemplates();
    localStorage.setItem("phylopanel.binary", state.binary);
  } catch (error) {
    state.schema = null;
    fail(String(error));
  }
}

/// Falls back to the gotree binary embedded in this executable, which is what makes the
/// download a single file with no setup.
async function useBundled(): Promise<boolean> {
  try {
    state.binary = await invoke<string>("bundled_tool");
  } catch (error) {
    fail(String(error));
    return false;
  }
  ($<HTMLInputElement>("binarypath")).value = state.binary;
  await reflect(false);
  return true;
}

async function chooseBinary() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t("dialog.chooseBinary"),
    filters: [{ name: t("dialog.executableFilter"), extensions: ["exe", "bin", ""] }],
  });
  if (typeof picked === "string") {
    state.binary = picked;
    ($<HTMLInputElement>("binarypath")).value = picked;
    await reflect(false);
  }
}

function leafMatches(node: TreeNode, query: string): boolean {
  if (!query) return true;
  const id = node.command?.id ?? node.fullPath.join(" ");
  const hay = `${id} ${node.command?.description ?? ""}`.toLowerCase();
  if (hay.includes(query)) return true;
  return node.children.some((child) => leafMatches(child, query));
}

function renderTree(query: string) {
  state.query = query;
  const host = $("cmdtree");
  host.replaceChildren();
  if (!state.schema) return;
  const needle = query.trim().toLowerCase();
  for (const node of buildTree(state.schema.commands)) {
    if (!leafMatches(node, needle)) continue;
    host.append(treeRow(node, needle, 0));
  }
}

function treeRow(node: TreeNode, query: string, depth: number): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "node";
  wrap.style.setProperty("--depth", String(depth));

  if (node.command && node.children.length === 0) {
    const button = document.createElement("button");
    button.className = "cmd";
    button.title = node.command.description;
    button.append(
      Object.assign(document.createElement("span"), { className: "cmdname", textContent: node.command.id }),
      Object.assign(document.createElement("span"), {
        className: "cmddesc",
        textContent: node.command.description.split("\n")[0] ?? "",
      })
    );
    button.addEventListener("click", () => addStep(node.command!));
    wrap.append(button);
    return wrap;
  }

  const head = document.createElement("button");
  head.className = "group";
  head.append(Object.assign(document.createElement("span"), { textContent: node.name }));
  if (node.command) {
    head.classList.add("dual");
    head.title = `${node.command.description}\n${t("tree.dualTitle")}`;
    head.append(
      Object.assign(document.createElement("i"), { className: "badge", textContent: t("tree.executable") })
    );
    head.addEventListener("click", () => addStep(node.command!));
  }
  wrap.append(head);

  const body = document.createElement("div");
  body.className = "children";
  for (const child of node.children) {
    if (query && !leafMatches(child, query)) continue;
    body.append(treeRow(child, query, depth + 1));
  }
  wrap.append(body);
  return wrap;
}

function addStep(command: CommandSpec) {
  state.steps.push({ command, values: {}, args: [] });
  state.editing = state.steps.length - 1;
  renderSteps();
  renderForm();
  refreshPreview();
}

function current(): Draft | null {
  return state.editing >= 0 ? state.steps[state.editing] ?? null : null;
}

function renderSteps() {
  const host = $("steps");
  host.replaceChildren();
  state.steps.forEach((draft, index) => {
    const li = document.createElement("li");
    li.className = index === state.editing ? "step active" : "step";

    const label = document.createElement("button");
    label.className = "steplabel";
    const given = materialFlags(draft.command, draft.values);
    const summary = Object.entries(given)
      .map(([k, v]) => (v === true ? `--${k}` : `--${k} ${v}`))
      .join("  ");
    label.append(
      Object.assign(document.createElement("span"), { className: "n", textContent: String(index + 1) }),
      Object.assign(document.createElement("code"), {
        textContent: draft.command.id + (draft.args.length ? ` ${draft.args.join(" ")}` : ""),
      }),
      Object.assign(document.createElement("span"), { className: "flags", textContent: summary })
    );
    label.addEventListener("click", () => {
      state.editing = index;
      renderSteps();
      renderForm();
    });

    const move = (delta: number) => {
      const to = index + delta;
      if (to < 0 || to >= state.steps.length) return;
      [state.steps[index], state.steps[to]] = [state.steps[to], state.steps[index]];
      state.editing = to;
      renderSteps();
      renderForm();
      refreshPreview();
    };
    const tools = document.createElement("div");
    tools.className = "step-tools";
    for (const [text, action] of [
      ["↑", () => move(-1)],
      ["↓", () => move(1)],
      ["×", () => {
        state.steps.splice(index, 1);
        state.editing = Math.min(state.editing, state.steps.length - 1);
        renderSteps();
        renderForm();
        refreshPreview();
      }],
    ] as [string, () => void][]) {
      const b = document.createElement("button");
      b.textContent = text;
      b.className = "ghost";
      b.addEventListener("click", action);
      tools.append(b);
    }

    li.append(label, tools);
    host.append(li);
  });

  if (state.steps.length === 0) {
    const empty = document.createElement("li");
    empty.className = "empty";
    empty.textContent = t("steps.empty");
    host.append(empty);
  }
}

function renderForm() {
  const host = $("form");
  const draft = current();
  if (!draft) {
    host.replaceChildren(
      Object.assign(document.createElement("p"), {
        className: "empty",
        textContent: t("form.pickStep"),
      })
    );
    $("formtitle").textContent = t("panel.params");
    return;
  }
  $("formtitle").textContent = t("form.paramsOf", { command: draft.command.id });
  host.replaceChildren(
    commandForm({
      command: draft.command,
      values: draft.values,
      args: draft.args,
      positional: state.pack?.positional[draft.command.id],
      onChange: (values, args) => {
        draft.values = values;
        draft.args = args;
        renderSteps();
        refreshPreview();
      },
      pickPath: async (flag, kind) => {
        const title =
          kind === "save" ? t("dialog.pickSave", { flag: flag.name }) : t("dialog.pickOpen", { flag: flag.name });
        const picked =
          kind === "save"
            ? await save({ title, defaultPath: flag.default ?? "" })
            : await open({ title, multiple: false, canChooseFiles: true });
        return typeof picked === "string" ? picked : null;
      },
    })
  );
}

function renderTemplates() {
  const host = $("templates");
  host.replaceChildren();
  for (const template of state.pack?.templates ?? []) {
    const button = document.createElement("button");
    button.className = "template";
    button.title = localize(template.descriptionZh, template.description);
    button.textContent = localize(template.nameZh, template.name);
    button.addEventListener("click", () => {
      if (!state.schema) return;
      const byId = new Map(state.schema.commands.map((c) => [c.id, c]));
      const drafts: Draft[] = [];
      for (const step of template.steps) {
        const command = byId.get(step.command);
        if (!command) continue;
        drafts.push({ command, values: { ...(step.flags as Values) }, args: step.args ?? [] });
      }
      if (drafts.length === 0) {
        setStatus(() => t("tpl.missing", { name: localize(template.nameZh, template.name) }), "warn");
        return;
      }
      state.steps = drafts;
      state.editing = 0;
      renderSteps();
      renderForm();
      refreshPreview();
    });
    host.append(button);
  }
}

function buildSteps(): StepSpec[] {
  return state.steps.map((draft) => {
    const flags = materialFlags(draft.command, draft.values);
    if (draft === state.steps[0] && state.inputMode === "file" && state.inputPath) {
      if (draft.command.flags.some((f) => f.role === "input") && flags.input === undefined) {
        flags.input = state.inputPath;
      }
    }
    return { command: draft.command.id, flags, args: draft.args };
  });
}

async function refreshPreview() {
  const host = $("preview");
  if (state.steps.length === 0) {
    host.textContent = "";
    return;
  }
  try {
    host.textContent = await invoke<string>("preview_command", { steps: buildSteps() });
  } catch {
    host.textContent = "";
  }
}

/** gotree emits bare width/height, so an <img> cannot scale it without a viewBox. */
function fitSvg(markup: string): string {
  const tag = /<svg\b[\s\S]*?>/.exec(markup)?.[0];
  if (!tag || tag.includes("viewBox")) return markup;
  const width = /width="([\d.]+)/.exec(tag)?.[1];
  const height = /height="([\d.]+)/.exec(tag)?.[1];
  if (!width || !height) return markup;
  const patched = tag.replace(
    />$/,
    ` viewBox="0 0 ${width} ${height}" preserveAspectRatio="xMidYMid meet">`
  );
  return markup.replace(tag, patched);
}

function renderOutput(result: RunResult) {
  const host = $("output");
  host.replaceChildren();

  for (const step of result.steps) {
    const row = document.createElement("div");
    row.className = step.ok ? "steprun ok" : "steprun fail";
    row.append(
      Object.assign(document.createElement("code"), { textContent: step.argv.join(" ") })
    );
    if (!step.ok) {
      row.append(
        Object.assign(document.createElement("span"), {
          className: "errmsg",
          textContent: step.message ?? t("out.execFailed"),
        })
      );
      const details = document.createElement("details");
      details.append(
        Object.assign(document.createElement("summary"), { textContent: t("out.rawStderr") }),
        Object.assign(document.createElement("pre"), { textContent: step.stderr })
      );
      row.append(details);
    }
    host.append(row);
  }

  const body = document.createElement("div");
  body.className = "artifact";
  const text = result.stdout.trimStart();
  if (result.isBinary && result.stdoutBase64) {
    const img = document.createElement("img");
    img.src = `data:image/png;base64,${result.stdoutBase64}`;
    img.alt = t("out.imageAlt");
    body.append(img);
  } else if (text.startsWith("<svg") || text.startsWith("<?xml")) {
    const url = URL.createObjectURL(new Blob([fitSvg(result.stdout)], { type: "image/svg+xml" }));
    const img = document.createElement("img");
    img.src = url;
    img.alt = t("out.treeAlt");
    body.append(img);
  } else {
    body.append(Object.assign(document.createElement("pre"), { textContent: result.stdout }));
  }
  host.append(body);
}

async function run() {
  if (state.busy) return;
  if (state.steps.length === 0) {
    setStatus(() => t("run.empty"), "warn");
    return;
  }
  state.busy = true;
  $("run").setAttribute("disabled", "disabled");
  setStatus(() => t("run.busy"));
  try {
    const result = await invoke<RunResult>("run_pipeline", {
      binary: state.binary,
      steps: buildSteps(),
      inputText: state.inputMode === "text" ? state.inputText : null,
    });
    state.last = result;
    renderOutput(result);
    $("preview").textContent = result.commandLine;
    $("saveout").removeAttribute("disabled");
    const failed = result.steps.filter((s) => !s.ok);
    if (failed.length) {
      const message = failed[0].message ?? "";
      setStatus(
        () => t("run.failed", { failed: failed.length, steps: result.steps.length, message }),
        "error"
      );
    } else {
      setStatus(() =>
        t("run.done", { steps: result.steps.length, ms: result.elapsedMs })
      );
    }
  } catch (error) {
    fail(String(error));
  } finally {
    state.busy = false;
    $("run").removeAttribute("disabled");
  }
}

async function saveOutput() {
  const result = state.last;
  if (!result) return;
  const target = await save({
    title: t("out.saveTitle"),
    defaultPath: result.isBinary
      ? "tree.png"
      : result.stdout.trimStart().startsWith("<svg")
        ? "tree.svg"
        : "result.txt",
  });
  if (typeof target !== "string") return;
  try {
    if (result.isBinary && result.stdoutBase64) {
      await invoke("write_base64", { path: target, data: result.stdoutBase64 });
    } else {
      await invoke("write_text", { path: target, text: result.stdout });
    }
  } catch (error) {
    fail(String(error));
    return;
  }
  setStatus(() => t("out.saved", { path: target as string }));
}

async function copyPreview() {
  const text = $("preview").textContent ?? "";
  if (!text) return;
  await navigator.clipboard.writeText(text);
  setStatus(() => t("out.copyDone"));
}

/** Everything that carries language-dependent text, rebuilt after a locale switch. */
function repaint() {
  renderTree(state.query);
  renderTemplates();
  renderSteps();
  renderForm();
  if (state.last) renderOutput(state.last);
  paintStatus();
}

function wirePreferences() {
  const langSelect = $<HTMLSelectElement>("lang");
  const themeSelect = $<HTMLSelectElement>("theme");
  langSelect.value = lang();
  themeSelect.value = themeChoice();

  langSelect.addEventListener("change", () => {
    applyLang(langSelect.value as Lang);
    repaint();
  });
  themeSelect.addEventListener("change", () => {
    applyTheme(themeSelect.value as ThemeChoice);
  });
}

function wire() {
  $("binarypath").addEventListener("change", async (e) => {
    const value = (e.target as HTMLInputElement).value.trim();
    if (!value || value === state.binary) return;
    state.binary = value;
    await reflect(false);
  });
  $("choose").addEventListener("click", chooseBinary);
  $("builtin").addEventListener("click", () => void useBundled());
  $("refresh").addEventListener("click", () => reflect(true));
  $("run").addEventListener("click", run);
  $("saveout").addEventListener("click", saveOutput);
  $("copy").addEventListener("click", copyPreview);
  $("clear").addEventListener("click", () => {
    state.steps = [];
    state.editing = -1;
    renderSteps();
    renderForm();
    refreshPreview();
  });
  $("search").addEventListener("input", (e) => renderTree((e.target as HTMLInputElement).value));

  const mode = (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.checked) return;
    state.inputMode = input.value as "file" | "text";
    $("inputfile").hidden = state.inputMode !== "file";
    $("inputtext").hidden = state.inputMode !== "text";
    refreshPreview();
  };
  for (const radio of document.querySelectorAll<HTMLInputElement>("input[name='inputmode']")) {
    radio.addEventListener("change", mode);
  }
  $("path").addEventListener("change", (e) => {
    state.inputPath = (e.target as HTMLInputElement).value;
    localStorage.setItem("phylopanel.inputPath", state.inputPath);
    refreshPreview();
  });
  $("tree").addEventListener("input", (e) => {
    state.inputText = (e.target as HTMLTextAreaElement).value;
    refreshPreview();
  });
  const preset = document.querySelector<HTMLInputElement>("input[name='inputmode']:checked");
  state.inputMode = (preset?.value as "file" | "text") ?? "file";
  $("inputfile").hidden = state.inputMode !== "file";
  $("inputtext").hidden = state.inputMode !== "text";
}

function start() {
  applyTheme(themeChoice());
  applyLang(lang());
  ($<HTMLInputElement>("binarypath")).value = state.binary;
  ($<HTMLInputElement>("path")).value = state.inputPath;
  wirePreferences();
  wire();
  renderSteps();
  renderForm();
  if (state.binary) void reflect(false);
  else void useBundled();
}

start();
