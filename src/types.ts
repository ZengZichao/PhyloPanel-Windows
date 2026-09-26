export type FlagKind = "bool" | "int" | "float" | "string" | "stringSlice" | "duration";
export type FlagRole = "input" | "output" | "format" | "file" | "other";

export interface FlagSpec {
  name: string;
  short?: string | null;
  kind: FlagKind;
  role: FlagRole;
  default?: string | null;
  description: string;
  choices?: string[] | null;
  inherited: boolean;
}

export interface CommandSpec {
  id: string;
  path: string[];
  group: string;
  description: string;
  flags: FlagSpec[];
}

export interface ToolSchema {
  binary: string;
  version: string;
  commands: CommandSpec[];
}

export interface PositionalSpec {
  label: string;
  labelZh?: string | null;
  help: string;
  helpZh?: string | null;
  minimum: number;
}

export interface StepSpec {
  command: string;
  flags: Record<string, unknown>;
  args?: string[];
}

export interface Template {
  name: string;
  nameZh?: string | null;
  description: string;
  descriptionZh?: string | null;
  steps: StepSpec[];
  terminal?: boolean;
}

export interface ToolPack {
  displayName: string;
  positional: Record<string, PositionalSpec>;
  templates: Template[];
  imageCommands: string[];
}

export interface StepResult {
  command: string;
  argv: string[];
  code: number | null;
  ok: boolean;
  stderr: string;
  message: string | null;
}

export interface RunResult {
  commandLine: string;
  stdout: string;
  stdoutBase64?: string | null;
  isBinary: boolean;
  elapsedMs: number;
  steps: StepResult[];
}

export interface Reflected {
  schema: ToolSchema;
  pack: ToolPack;
  fromCache: boolean;
}

/** Rebuilds the cobra command tree from flat command paths. */
export interface TreeNode {
  name: string;
  fullPath: string[];
  children: TreeNode[];
  command?: CommandSpec;
}

export function buildTree(commands: CommandSpec[]): TreeNode[] {
  const root: TreeNode[] = [];
  for (const command of commands) {
    let level = root;
    let node: TreeNode | undefined;
    command.path.forEach((segment, index) => {
      const last = index === command.path.length - 1;
      node = level.find((n) => n.name === segment);
      if (!node) {
        node = {
          name: segment,
          fullPath: command.path.slice(0, index + 1),
          children: [],
        };
        level.push(node);
      }
      if (last) node.command = command;
      level = node.children;
    });
  }
  const sort = (nodes: TreeNode[]) => {
    nodes.sort((a, b) => a.name.localeCompare(b.name));
    nodes.forEach((n) => sort(n.children));
  };
  sort(root);
  return root;
}

/** A flag only reaches the command line when it deviates from the CLI default. */
export function materialFlags(
  command: CommandSpec,
  values: Record<string, unknown>
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const flag of command.flags) {
    const value = values[flag.name];
    if (value === undefined || value === null || value === "") continue;
    if (flag.kind === "bool") {
      if (value === true) out[flag.name] = true;
      continue;
    }
    if (flag.kind === "stringSlice") {
      const items = Array.isArray(value) ? value : String(value).split(",").map((s) => s.trim());
      const kept = items.filter((s) => s !== "");
      if (kept.length) out[flag.name] = kept;
      continue;
    }
    if (flag.default !== null && flag.default !== undefined && String(value) === flag.default) {
      continue;
    }
    out[flag.name] = value;
  }
  return out;
}
