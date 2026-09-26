import { localize, t } from "./i18n";
import type { CommandSpec, FlagSpec, PositionalSpec } from "./types";

export type Values = Record<string, unknown>;

export interface FormOptions {
  command: CommandSpec;
  values: Values;
  args: string[];
  positional?: PositionalSpec;
  onChange: (values: Values, args: string[]) => void;
  pickPath: (flag: FlagSpec, kind: "open" | "save") => Promise<string | null>;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Record<string, unknown> = {},
  children: (Node | string)[] = []
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (key === "class") node.className = String(value);
    else if (key === "dataset") Object.assign(node.dataset, value as Record<string, string>);
    else if (value !== undefined && value !== null) node.setAttribute(key, String(value));
  }
  for (const child of children) node.append(child);
  return node;
}

const ROLE_ORDER: Record<string, number> = { input: 0, output: 1, format: 2, file: 3, other: 4 };

function sorted(flags: FlagSpec[]): FlagSpec[] {
  return [...flags].sort(
    (a, b) => (ROLE_ORDER[a.role] ?? 4) - (ROLE_ORDER[b.role] ?? 4) || a.name.localeCompare(b.name)
  );
}

function parse(kind: FlagSpec["kind"], raw: string): unknown {
  if (kind === "int" || kind === "float") {
    if (raw.trim() === "") return "";
    const n = Number(raw);
    return Number.isFinite(n) ? n : raw;
  }
  return raw;
}

function control(
  flag: FlagSpec,
  value: unknown,
  opts: FormOptions,
  write: (v: unknown) => void
): HTMLElement {
  if (flag.kind === "bool") {
    const label = el("label", { class: "switch" }, [
      el("input", {
        type: "checkbox",
        ...(value === true ? { checked: "checked" } : {}),
      }),
      el("span", {}, [t("form.enable")]),
    ]);
    label.querySelector("input")!.addEventListener("change", (e) =>
      write((e.target as HTMLInputElement).checked)
    );
    return label;
  }

  if (flag.choices && flag.choices.length > 0) {
    const select = el("select", { "aria-label": flag.name }, [
      el("option", { value: "" }, [t("form.defaultOption")]),
      ...flag.choices.map((choice) =>
        el("option", { value: choice, ...(value === choice ? { selected: "selected" } : {}) }, [
          choice,
        ])
      ),
    ]);
    select.addEventListener("change", () => write(select.value));
    return select;
  }

  const isPath = flag.role === "input" || flag.role === "output" || flag.role === "file";
  const input = el("input", {
    type: flag.kind === "int" || flag.kind === "float" ? "number" : "text",
    value: value === undefined || value === null ? "" : String(value),
    placeholder: flag.default ? t("form.default", { value: flag.default }) : "",
    step: flag.kind === "float" ? "any" : undefined,
    spellcheck: "false",
  });
  input.addEventListener("input", () => write(parse(flag.kind, (input as HTMLInputElement).value)));

  if (!isPath) return input;

  const kind = flag.role === "output" ? "save" : "open";
  const browse = el("button", { type: "button", class: "ghost" }, [t("form.browse")]);
  browse.addEventListener("click", async () => {
    const picked = await opts.pickPath(flag, kind);
    if (picked) {
      (input as HTMLInputElement).value = picked;
      write(picked);
    }
  });
  return el("div", { class: "inline" }, [input, browse]);
}

export function commandForm(opts: FormOptions): HTMLElement {
  const { command, values, positional } = opts;
  const form = el("div", { class: "form" });
  const patch = (name: string, value: unknown) =>
    opts.onChange({ ...values, [name]: value }, opts.args);

  const own = sorted(command.flags.filter((f) => !f.inherited));
  const shared = sorted(command.flags.filter((f) => f.inherited));

  const section = (title: string, hint: string, flags: FlagSpec[], collapsible = false) => {
    if (flags.length === 0) return;
    let host: HTMLElement = form;
    if (collapsible) {
      const box = el("details", { class: "collapse" });
      const summary = el("summary", { title: hint }, [`${title} (${flags.length})`]);
      box.append(summary);
      form.append(box);
      host = box;
    } else {
      host.append(el("h4", { title: hint }, [title]));
    }
    for (const flag of flags) {
      const meta = el("div", { class: "meta" }, [
        el("code", {}, [flag.short ? `-${flag.short}, --${flag.name}` : `--${flag.name}`]),
        el("span", { class: "kind" }, [flag.kind]),
        flag.default ? el("span", { class: "default" }, [t("form.default", { value: flag.default })]) : "",
      ]);
      const described = flag.description ? el("p", { class: "desc" }, [flag.description]) : "";
      host.append(
        el(
          "div",
          { class: "row" },
          [meta, control(flag, values[flag.name], opts, (v) => patch(flag.name, v)), described]
        )
      );
    }
  };

  section(t("form.own"), t("form.ownHint"), own);
  section(t("form.shared"), t("form.sharedHint"), shared, true);

  if (positional) {
    const input = el("input", {
      type: "text",
      value: opts.args.join(" "),
      placeholder: localize(positional.helpZh, positional.help),
      spellcheck: "false",
    });
    input.addEventListener("input", () =>
      opts.onChange(values, input.value.split(/\s+/).filter(Boolean))
    );
    form.append(
      el("h4", { title: t("form.positionalHint") }, [
        t("form.positional", { label: localize(positional.labelZh, positional.label) }),
      ]),
      el("div", { class: "row" }, [
        el("p", { class: "desc" }, [localize(positional.helpZh, positional.help)]),
        input,
      ])
    );
  }

  if (own.length === 0 && shared.length === 0 && !positional) {
    form.append(el("p", { class: "empty" }, [t("form.noFlags")]));
  }
  return form;
}
