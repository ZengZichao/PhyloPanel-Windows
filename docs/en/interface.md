# Interface tour

The window has a header, a status line, and three columns.

```
┌────────────────────────────────────────────────────────────────────────────┐
│ Executable [path] Browse… Use bundled GoTree Re-reflect │ presets │ lang │ theme │
├──────────────────────────── status line ───────────────────────────────────┤
│ Commands      │  Input                    │  Workflow                       │
│  (search +    │  Parameters               │  Equivalent command line        │
│   tree)       │                           │  Result                         │
└───────────────┴───────────────────────────┴─────────────────────────────────┘
```

## Header

| Control | What it does |
|---|---|
| **Executable** | Full path to the CLI being driven. Editable: press Enter or click away and the panel re-reflects that binary. |
| **Browse…** | Native file picker for the same thing. |
| **Use bundled GoTree** | Extracts (or reuses) the GoTree binary embedded in the exe and reflects it. This is what runs automatically on first launch. |
| **Re-reflect commands** | Ignores the cache and re-reads every `--help`. Use it after replacing the CLI binary. |
| **Preset workflows** | Buttons supplied by the tool pack; each fills the workflow with a ready-made chain. Hover for a one-line description. |
| **Language** | 中文 / English. Applies instantly and is remembered. |
| **Theme** | Follow system / Light / Dark. Also repaints the Windows title bar. |

## Status line

A single line of feedback, colour-coded:

| Appearance | Meaning |
|---|---|
| Grey on light background | Informational: progress, reflection results, success. |
| Amber | Warning: nothing ran, e.g. no executable selected, empty workflow, a preset that does not fit the current CLI version. |
| Red | Failure: the backend refused or a step errored. The message includes the reason. |

Typical messages: `Reflecting the command tree…`, `Executable not found: C:\…`,
`Done · 2 steps · 118 ms`, `1 of 2 steps failed · <tool's own message>`, `Saved to <path>`.

## Column 1 — Commands

A search box plus the reflected command tree, grouped by command family and sorted
alphabetically.

- **Click a leaf** to append it to the workflow.
- **Dashed border with a `runnable` badge** marks a *dual* command: it both runs on its own
  and has sub-commands (`cut`, `resolve`, `stats`, `prune`, …). Clicking the group heading
  adds the command itself; clicking a child adds the sub-command.
- **The grey text** after each command id is the tool's own short description, taken from
  `--help`. It stays in English because it is the CLI's text, not the panel's.
- **Search** matches the command id and its description, and keeps parent groups visible so
  you can see where a hit lives.

Coverage for GoTree 0.5.2: 82 runnable commands in 35 groups, 213 command-specific flags.

## Column 2 — Input and Parameters

**Input** has two mutually exclusive modes:

- **Tree file (passes --input)** — a path. If the first workflow step owns an input-role flag
  and you left it empty, the panel fills it with this path.
- **Paste directly** — a Newick string written to the first process's standard input.

**Parameters** renders a form for whichever workflow step is selected. Its heading shows the
command, e.g. `Parameters · draw svg`.

Each flag row contains:

| Part | Example | Notes |
|---|---|---|
| Flag name | `-H, --height` | Short and long forms as the CLI declares them. |
| Kind chip | `int` | One of `bool`, `int`, `float`, `string`, `stringSlice`, `duration`. |
| Default chip | `default 200` | The CLI's own default. Leaving the field at that value emits nothing. |
| Control | number box, checkbox, dropdown, text box | Enumerated flags become dropdowns; path-like flags get a **Browse…** button. |

Flags are grouped into **Command flags** (this command's own) and a collapsed **Shared flags**
section (global flags inherited from the root). Shared flags are usually noise; open the
section only when you need them.

Some commands take bare arguments that cobra never advertises. For those, a **Positional
arguments** section appears with a text box — space-separated values. In GoTree 0.5.2 that
affects exactly four commands: `reroot outgroup`, `collapse clade`, `prune`, `upload itol`.

## Column 3 — Workflow, Command line, Result

**Workflow** lists the steps in execution order. The selected step is outlined.

| Button | Effect |
|---|---|
| ↑ / ↓ | Reorder the step. |
| × | Remove the step. |
| Step label | Select the step, which loads its parameters into column 2. |
| **Run** | Execute the whole chain. Disabled while a run is in flight. |
| **Clear** | Empty the workflow. Input text and paths are kept. |

**Equivalent command line** shows the shell rendering live, before you run anything. **Copy**
puts it on the clipboard.

**Result** shows one line per step — green for success, red with the tool's message and a
collapsible raw-stderr section for failure — followed by the final output:

- A tree or table renders as monospaced text.
- An SVG drawing renders as an image. GoTree emits `width`/`height` without a `viewBox`, so
  the panel injects one; the image is shown at its natural size inside a scrollable frame,
  because the CLI hard-codes 8 px label text that becomes unreadable when scaled down.
- A PNG (`draw png`) is decoded and shown directly.
- **Save output…** suggests `tree.svg`, `tree.png` or `result.txt` based on what you got.

## Keyboard notes

There are no custom shortcuts yet. Standard ones work: `Tab` between controls, `Enter` in the
executable field commits a new path, `Ctrl+F` does *not* open the panel's search box — click
it instead.
