# Command coverage

## How the command tree is discovered

PhyloPanel does not contain a list of GoTree commands. On first use it *reflects* the binary:

1. Run the tool with no arguments and read the `Available Commands` sections.
2. Recurse into every command group.
3. For each candidate, run `<tool> <path> --help` and parse the usage line, the flags block
   and the short description.
4. Classify each command as a leaf, a group, or **dual** (both — GoTree's `cut`, `resolve`
   and `stats` are executable *and* have children).
5. Type each flag from its placeholder (`--height int` → integer, `--support float`,
   `--labels stringSlice`, a bare `--circular` → boolean), and pull the default out of the
   help text.
6. Assign each flag a semantic role — `input`, `output`, `format`, `file`, `other` — which is
   what drives file pickers and the automatic `--input` injection.

Roles matter because the CLIs are inconsistent: `-i/--input` is a global flag on `draw svg`
but a private one on `acr`, and `reroot outgroup` says `--tip-file` where `prune` says
`--tipfile`. Keying the UI on meaning rather than spelling is what keeps one form builder
working across tools.

## Caching

Reflection means roughly one process per command, so the result is cached as JSON, keyed by
the binary's file name **and its modification time**. Replace the CLI and the cache entry no
longer matches, so the next launch re-reflects automatically.

**Re-reflect commands** in the header forces a fresh reflection and rewrites the cache. Use it
if you edited the binary in place without changing its timestamp.

## What GoTree 0.5.2 exposes

82 runnable commands across 35 top-level groups, with 213 command-specific flags. Counts and
descriptions below were read from the reflected schema, not from documentation.

| Group | Commands | Short description |
|---|---|---|
| `acr` | 1 | Reconstructs most parsimonious ancestral characters |
| `annotate` | 1 | Annotates internal branches with given data |
| `asr` | 1 | Reconstructs most parsimonious ancestral sequences |
| `brlen` | 8 | Branch-length arithmetic: add, clear, cut, round, scale, set, setmin, setrand |
| `collapse` | 6 | Collapse clades by tips, support, depth, length, name, single child |
| `comment` | 2 | Add or remove node/tip/edge comments |
| `compare` | 4 | Compare two trees (Robinson-Foulds and related) |
| `compute` | 7 | bipartitiontree, consensus, edgetrees, mutations, roccurve, `support fbp`, `support tbe` |
| `cut` | 2 | Cut the tree (dual: runs on its own too) |
| `divide` | 1 | Split an input tree file into several files |
| `download` | 3 | Download a tree image or file from iTOL |
| `draw` | 4 | `draw cyjs`, `draw png`, `draw svg`, `draw text` |
| `generate` | 6 | Generate random trees (balanced, caterpillar, …) |
| `graft` | 1 | Graft one tree onto another at a given tip |
| `labels` | 1 | List all tip labels |
| `ltt` | 1 | Lineage-through-time data |
| `matrix` | 1 | Print the distance matrix |
| `merge` | 1 | Merge two rooted trees under a new root |
| `nni` | 1 | Generate all NNI neighbours |
| `prune` | 1 | Remove tips (takes positional arguments) |
| `reformat` | 3 | Reformat into Newick and relatives |
| `rename` | 1 | Rename nodes or tips |
| `repopulate` | 1 | Re-add tips sharing identical sequences |
| `reroot` | 2 | `reroot midpoint`, `reroot outgroup` |
| `resolve` | 2 | Resolve multifurcations (dual) |
| `rotate` | 2 | Rotate children of internal nodes |
| `rtt` | 1 | Root-to-tip regression |
| `sample` | 1 | Sub-sample trees from a file |
| `shuffletips` | 1 | Shuffle tip names |
| `stats` | 7 | Tree statistics, including `stats monophyletic` (dual) |
| `subtree` | 1 | Extract a subtree by node name |
| `support` | 4 | Support-value maintenance |
| `unroot` | 1 | Unroot the tree |
| `upload` | 1 | `upload itol` (takes positional arguments) |
| `version` | 1 | Print the tool version |

Two documentation traps worth knowing: `monophyletic` lives **under** `stats`
(`gotree stats monophyletic`), and the four commands listed above read bare arguments, which
`--help` never mentions.

## Driving GoAlign or any other cobra CLI

Point **Executable** at `goalign.exe` and the same reflection lists every GoAlign command and
flag, and pipelines between them work. **But GoAlign is not embedded in the download, and it
has no tool pack yet**, so what you lose is the curated layer: no preset workflow buttons, no
positional-argument boxes, and `imageCommands` falls back to GoTree's `draw svg/png/cyjs` names
so picture detection will not match GoAlign's output. Build each step by hand, or write the
pack described below.

The mechanism is generic; what is tool-specific lives in the *tool pack*.

### The tool pack

`src-tauri/toolpacks/<name>.json` is selected by matching the binary's file name and supplies
what reflection cannot discover:

| Field | Purpose |
|---|---|
| `displayName` | Name shown in the status line |
| `positional` | Per-command label and help for bare arguments, plus `nameZh`/`helpZh` for the Chinese UI |
| `templates` | Preset workflows, with `name`/`nameZh` and `description`/`descriptionZh` |
| `imageCommands` | Which commands return a picture rather than a tree or table |

An unknown binary still works fully — every command and flag is reflected — it just gets no
presets and no positional hints. Adding those is a JSON edit, not a code change.

### Adding a pack for a new tool

1. Create `src-tauri/toolpacks/<tool>.json`.
2. Load it in `pack_for()` in `src-tauri/src/toolpack.rs` by matching the binary's stem, the
   same way `gotree.json` is matched.
3. Rebuild. Reflection picks up the commands; the pack adds the curated layer.

## Version drift

Because the tree is read from the binary at run time, an upgraded CLI appears in the panel
with no changes here. If an upgrade renames a command that a preset depends on, the preset
reports which command went missing rather than running a broken chain.
