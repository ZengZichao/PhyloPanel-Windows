# Workflows and pipelines

## The pipeline model

A workflow is an ordered list of commands. PhyloPanel starts one child process per step and
wires them together with real OS pipes: step *n*'s stdout becomes step *n+1*'s stdin. Nothing
is buffered to disk, and no intermediate tree file appears anywhere.

```
input ──stdin──► [ step 1 ] ──stdout──► [ step 2 ] ──stdout──► [ step 3 ] ──► Result
```

Consequences worth internalising:

- **Order is semantics.** `reroot midpoint | draw svg` draws a rerooted tree; the reverse
  order is not the same operation.
- **A step that consumes the tree ends the chain.** `stats`, `compare trees`, `matrix`,
  `labels` print a report instead of emitting a tree, so anything after them receives no
  input. The presets that use them are marked as terminal.
- **Every step runs even if an earlier one fails**, because the processes are launched
  together. If step 1 dies, step 2 sees an empty stdin and usually errors too; the result
  panel shows which step said what.
- **Exit status per step is tracked separately**, so a green line and a red line can coexist.

## Building a chain

1. Click commands in the **Commands** tree; each one is appended.
2. Click a step's label to edit its parameters.
3. Use ↑ / ↓ to reorder, × to delete.
4. **Run**.

**Clear** empties the workflow but keeps your input path or pasted text, so you can rebuild a
chain without re-supplying the tree.

## How flags become a command line

The panel deliberately emits as little as possible, so the generated line stays readable and
identical to what an expert would type.

| You set | Emitted | Why |
|---|---|---|
| A boolean checkbox, off | *nothing* | False is the default. |
| A boolean checkbox, on | `--circular` | |
| A number equal to the CLI default | *nothing* | Restating a default adds noise. |
| A number different from the default | `--height 400` | |
| An empty text field | *nothing* | |
| A list field with `a,b,c` | `--labels a,b,c` | `stringSlice` flags are joined with commas. |
| Positional arguments `A B C` | `… prune A B C` | Appended after the flags. |

So an empty form is not "no options applied" — it means "run this command exactly as the CLI
would with its own defaults".

## Automatic input injection

With **Tree file** selected, the path is added to the first step automatically *only if* all
of these hold:

1. the step owns a flag whose role is `input`, and
2. you did not fill that flag yourself.

If you typed a value into the step's own `--input` field, yours wins. With **Paste directly**,
no `--input` is added at all; the text is written to the first process's stdin.

## Presets

Presets come from the tool pack, not from hard-coded UI, and they are per-tool. The bundled
GoTree pack ships five:

| Preset | Chain |
|---|---|
| Mid-point reroot then draw | `reroot midpoint \| draw svg --width 900 --height 900` |
| Collapse weak branches then draw | `collapse support --support 0.7 \| draw svg --width 900 --height 900 --with-branch-support` |
| Radial layout | `draw svg --radial --width 900 --height 900` |
| Tree statistics *(terminal)* | `stats` |
| Robinson-Foulds comparison *(terminal)* | `compare trees --rf` |

Pressing a preset replaces the current workflow. If a preset references commands that the
loaded CLI version does not have, the panel warns in the status line instead of loading a
half-built chain.

## Positional arguments

Cobra describes flags in `--help` but not bare arguments, so those cannot be reflected — they
are curated in the tool pack. Four GoTree commands need them:

| Command | Expects |
|---|---|
| `reroot outgroup` | tip names defining the outgroup (or use `--tip-file`) |
| `collapse clade` | tip names defining the clade to collapse |
| `prune` | tip names to remove |
| `upload itol` | tree files to upload |

When one of these is selected, the form shows a **Positional arguments** box; separate values
with spaces. Note that GoTree itself is inconsistent here — `reroot outgroup` reads
`--tip-file` while `prune` reads `--tipfile` — which is exactly why the panel keys its forms
on semantic role rather than on flag names.

## Recipes

**A publication-ready radial SVG from a file**

```
unroot | reroot midpoint | collapse support --support 0.95 | draw svg --radial --width 1200 --height 1200
```

**Compare two topologies and keep the numbers**

```
compare trees --rf --input reference.tre
```

**Sub-sample a Bayesian tree set, then summarise it**

```
sample --number 100 | stats
```

**Rename tips from a mapping file and draw**

```
rename --node-file map.txt | draw text --width 60
```

Anything you can build in a terminal you can build here, and the equivalent command line is
the check: if the line looks wrong, the run will be wrong.
