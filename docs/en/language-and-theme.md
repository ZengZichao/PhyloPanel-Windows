# Language and theme

Both controls live at the right end of the header, and both take effect instantly with no
restart.

## Language

**Options:** 中文 and English. The names are shown in their own language on purpose, so the
list is readable whichever language is currently active.

**What the switch covers:**

- Every header control, label, hint and placeholder.
- All three panel headings and the workflow, command-line and result sections.
- Flag form text: section titles, the `Enable` checkbox, `(default)` options, `default 200`
  chips, `Browse…`, the positional-arguments heading and its explanation.
- Status messages, warnings and errors — including errors raised in the Rust backend.
- Empty-state hints and image alt text.
- Preset workflow names and descriptions, and the positional-argument help text, which are
  curated per tool in `src-tauri/toolpacks/*.json`.

**What it deliberately does not translate:** command ids and flag help strings. Those are read
live from the target CLI's own `--help` output, so they belong to GoTree or GoAlign, not to
PhyloPanel. Translating them in the panel would mean shipping a private copy of someone
else's documentation that silently goes stale on every upstream release. Everything the panel
itself authored is bilingual.

**How the initial value is chosen:** if you have never touched the control, the panel reads
the system language and starts in Chinese for any `zh-*` locale, English otherwise. Once you
choose, that choice wins.

**Where it is stored:** `phylopanel.lang` in the app's local storage, inside the WebView2 user
profile under `%LOCALAPPDATA%\app.phylo.panel\`. It survives restarts and app updates.

**Adding a third language** takes three steps: add a message table in `src/i18n.ts`, extend the
`Lang` union, and add the corresponding fields to the tool pack JSON. The `missingKeys()`
helper in `src/i18n.ts` reports any key defined in one language but not another.

## Theme

**Options:** Follow system, Light, Dark.

- **Follow system** resolves through the `prefers-color-scheme` media query and keeps
  tracking: change the Windows setting while the app is open and the panel follows.
- **Light** and **Dark** pin the choice regardless of the OS.

The selection also calls the window manager so the **native Windows title bar** matches the
content instead of staying white above a dark window. In a plain browser (`npm run dev`
without Tauri) that call is skipped silently.

**How it is implemented:** every colour in `src/styles.css` is a CSS custom property. The
light palette is declared on `:root`, and a `[data-theme="dark"]` block overrides the same
names. No rule anywhere hard-codes a colour, so a new palette is a token edit, not a search
and replace. The resolved value is written to `<html data-theme>` and `color-scheme` follows
it, which is what makes native scrollbars and form controls match.

**Contrast:** both palettes were measured against WCAG text contrast, and the interface
colours clear 4.5:1 for body text, secondary text, headings, group rows, buttons and the
command-line block.

**One deliberate exception:** the drawing frame keeps a white background in the dark theme.
GoTree's SVG output is black-on-transparent with hard-coded 8 px labels, so inverting the
frame would render the tree invisible. The *chrome* goes dark; the *figure* stays on paper.

**Persistence:** `phylopanel.theme` in local storage, same profile folder. An unmodified value
means Follow system.

**Known limitation:** because the theme is applied by JavaScript a moment after the document
is parsed, a window whose explicit choice differs from the system setting can flash in the
system theme for a fraction of a second on launch. Follow system never flashes.

## Both together

Language and theme are independent, so all six combinations work, and each is remembered
separately. The screenshots in the [README](../../README.md) show English/Light and
Chinese/Dark.
