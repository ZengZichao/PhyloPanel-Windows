# Troubleshooting

## Startup and reflection

**Status says `Executable not found: <path>`.** The **Executable** field points at a file that
no longer exists — usually a deleted unpacked GoTree, or a path you typed that lost its
extension. Press **Use bundled GoTree** to reset to the embedded binary.

**Status says `Reflecting the command tree failed`.** The binary is not a cobra CLI, is
corrupted, or was blocked by antivirus on first run. Test it yourself:

```
"C:\path\to\tool.exe" --help
```

If that prints a command list, the panel can reflect it; if not, the tool is the problem.

**The command tree is empty or stale after you replaced the CLI.** Reflection is cached per
binary name *and modification time*. If your copy tool preserved the original timestamp, the
cache still looks valid. Press **Re-reflect commands** to force a rebuild.

**First launch is slow, then fast forever.** Normal: the embedded GoTree (~20 MB) is unpacked
once and every command's `--help` is read once. Both results are cached.

## Running

**`There are no steps to run`.** The workflow is empty. Click a command first.

**`The previous step produced no output to pipe`.** A step was expected to hand its stdout to
the next process but its output stream could not be wired. Usually means an earlier step died
immediately — look at the red step above it.

**A step is red but the tool works in a terminal.** Open the collapsible **Raw stderr** under
that step. Two usual causes:

- **A missing positional argument.** `prune`, `reroot outgroup`, `collapse clade` and
  `upload itol` need bare arguments that no flag can supply; fill the
  **Positional arguments** box.
- **A required flag you left at its default.** The panel omits flags that merely restate the
  CLI default, which is correct for a normal run but not for a command that requires the flag
  to be present. Set it to a value that differs from the default.

**The error message is only `Not group given` or similar.** That is GoTree's own wording,
passed through verbatim — the panel does not invent it. Search the upstream documentation for
the command.

**The drawing is blank or tiny.** GoTree hard-codes 8 px label text and emits `width`/`height`
without a `viewBox`. The panel injects a `viewBox` and displays the image at natural size in a
scrollable frame, so increase `--width` / `--height` on the `draw svg` step for a bigger tree
rather than zooming the result.

**Nothing happens when I press Run.** A run is already in flight — the button is disabled until
it finishes. Check the status line for `Running…`.

## Output

**`Could not write the file`.** The target is open in another program (common with SVG/PNG
viewers and Excel holding `.txt`), or the folder is read-only. Close the viewer and save again.

**Saved SVG looks right but a text editor shows no `viewBox`.** Correct — the `viewBox` the
panel adds is for on-screen display only. The saved file is GoTree's own output, byte for byte.

**The PNG step produced text instead of a picture.** `draw png` must be the last step and its
output must be captured; if you chained something after it, that step received binary garbage.

## Interface

**The text is Chinese and I want English (or the reverse).** Header → **Language** → choose.
It applies instantly and is remembered. See [Language and theme](language-and-theme.md).

**Dark mode shows a white title bar briefly at startup.** The theme is applied by script just
after the document loads; with an explicit choice that contradicts the system setting there
can be a short flash. **Follow system** never flashes.

**The tree drawing stays on a white background in dark mode.** Intentional: GoTree's SVG is
black-on-transparent, so a dark frame would make it invisible.

**The layout looks cramped in English.** English labels are longer than Chinese ones. The
window remembers no size, so drag it wider; the minimum is 1000 × 660.

## Environment

**"This app can't run on your PC" or an instant crash.** Almost always a missing WebView2
Runtime on an old Windows 10. Install the Evergreen runtime from Microsoft and retry.

**Antivirus quarantined the exe.** The app writes an executable into its own app-data folder
on first launch, which some scanners flag. Restore the quarantine entry and add an exclusion
for `%APPDATA%\app.phylo.panel\`.

**No network access is required.** The app reflects a local binary and never phones home.
GoTree's own `download`/`upload` commands do use the network, but only when you run them.

## Collecting details when reporting a bug

Include: the PhyloPanel version, the CLI and version in the status line, the exact
**Equivalent command line**, the **Raw stderr** of the failing step, and — for layout or
language problems — which language and theme were active.
