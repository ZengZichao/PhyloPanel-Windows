# Building from source

## Toolchain

| Component | Version | Notes |
|---|---|---|
| Rust | stable, MSVC host (`x86_64-pc-windows-msvc`) | `rustup show` should list the MSVC toolchain |
| C++ build tools | Visual Studio 2022 Build Tools | the *Desktop development with C++* workload; needed for the `link.exe` Rust uses |
| Node.js | 18 or newer | 24 tested |
| WebView2 Runtime | Evergreen | present by default on Windows 11 |
| Git | any | only for cloning |

```bash
git clone https://github.com/ZengZichao/PhyloPanel-Windows.git
cd PhyloPanel-Windows
npm install
```

## Development loop

Two processes are involved: Vite serves the front end, and the Rust host window loads it.

```bash
# terminal 1
npm run dev            # vite on http://localhost:1420

# terminal 2
cd src-tauri
cargo run              # debug host, reads the devUrl from tauri.conf.json
```

The debug build points at the dev URL, so front-end edits hot-reload; Rust changes need
`cargo run` again. Alternatively `npx tauri dev` starts both.

## Tests

```bash
cd src-tauri
cargo test --release
```

Eight tests: five pure unit tests over help parsing, flag materialisation and shell quoting,
and three end-to-end tests that drive a **real** GoTree binary through the pipeline. The
end-to-end ones are skipped unless the binary is supplied:

```bash
GOTREE_BIN=/path/to/gotree.exe cargo test --release
```

`src-tauri/tools/gotree.exe` works as the target, so the suite can be run straight from a
fresh clone.

The front end has no test runner; `npm run build` runs `tsc --noEmit` first, which is the
compile-time gate.

## Release build

```bash
npm run build                 # tsc --noEmit, then vite build → dist/
npx tauri build --no-bundle   # → src-tauri/target/release/phylopanel.exe
```

`--no-bundle` skips the NSIS installer and leaves the single self-contained exe (~29 MB),
which is the shape this app is designed to be distributed in. Without the flag you get an
installer under `src-tauri/target/release/bundle/`.

### What ends up inside the exe

- The Rust binary with the compiled front end embedded from `dist/`.
- `src-tauri/tools/gotree.exe`, included with `include_bytes!` in `src/bundle.rs` and unpacked
  on first launch into `%APPDATA%\app.phylo.panel\tools\`, named after an FNV-1a fingerprint of
  its contents. It is written to a `.part` file and renamed, so a crash mid-install can never
  leave a half-written tool that looks valid.

Because the tool is embedded, `src-tauri/tools/gotree.exe` **must be committed** even though
other `.exe` files are ignored — see the `!src-tauri/tools/*.exe` exception in `.gitignore`.

### Rebuilding the embedded GoTree

Only needed to track an upstream release:

```bash
git clone https://github.com/choishingwan/GoTree
cd GoTree
go build -trimpath -ldflags="-s -w -X github.com/evolbioinfo/gotree/cmd.Version=v0.5.2" -o ../PhyloPanel-Windows/src-tauri/tools/gotree.exe .
```

The `-X ...cmd.Version` flag matters: without it `gotree version` prints nothing and the
panel's status line loses its version field. Upstream's own Makefile injects it the same way.

In mainland China you may need `GOPROXY=https://goproxy.cn,direct`. Drop the result in, delete
the reflection cache, and rebuild the panel.

## Adding a language or a theme

- **Language:** add a message table in `src/i18n.ts`, widen the `Lang` union, and add the
  matching `*Zh`-style fields to the tool pack. `missingKeys()` lists keys defined in one
  table but not another.
- **Theme:** add a `[data-theme="…"]` block in `src/styles.css` overriding the same custom
  properties, then extend `ThemeChoice` in `src/theme.ts` and the `<select id="theme">` options
  in `index.html`. No rule hard-codes a colour, so a new palette is a token edit.

## Repository layout

```
src/                 front end
  i18n.ts            zh/en tables, t(), DOM localisation, backend error decoding
  theme.ts           theme resolution, persistence, title-bar sync
  main.ts            state, rendering, IPC calls
  controls.ts        flag form builder
  types.ts           shared types, command-tree builder, flag materialisation
index.html           skeleton with data-i18n markers
src-tauri/
  src/lib.rs         module wiring and the E#<key>|<detail> error helper
  src/schema.rs      --help parsing into a typed command schema
  src/runner.rs      process pipeline, argv construction, shell quoting
  src/commands.rs    the six IPC commands
  src/bundle.rs      embedding and unpacking the CLI
  src/toolpack.rs    tool pack loading and matching
  toolpacks/         per-tool curated JSON
  capabilities/      Tauri permission set (note core:window:allow-set-theme)
  tauri.conf.json    window, CSP, build hooks
docs/en, docs/zh     this documentation
samples/demo.tre     example input
```

## Troubleshooting the build

**Renaming the project directory breaks the build.** Cargo caches absolute paths in
`src-tauri/target`; delete that folder and rebuild.

**`link.exe` not found, or a panic about the MSVC toolchain.** Install or repair the
Visual Studio 2022 Build Tools C++ workload, and make sure the Rust host triple is
`x86_64-pc-windows-msvc`.

**The window opens blank.** Vite is not running, or is not on port 1420. Start `npm run dev`
first, or use a release build which reads embedded assets instead of the dev URL.

**The app starts but the command tree is empty.** The embedded tool failed to unpack — check
`%APPDATA%\app.phylo.panel\tools\` and the status line message.
