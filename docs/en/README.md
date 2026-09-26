# PhyloPanel documentation

Detailed usage documentation. Read the page that matches what you are doing; each page
stands on its own.

| Page | Read it when |
|---|---|
| [Getting started](getting-started.md) | You have just downloaded the exe and want your first tree drawn |
| [Interface tour](interface.md) | You want to know what every control, badge and status message means |
| [Workflows and pipelines](workflows.md) | You want to chain commands, or understand why a flag did or did not reach the command line |
| [Language and theme](language-and-theme.md) | You want to switch between Chinese and English, or light and dark |
| [Command coverage](command-reference.md) | You want to know what is reflected, how caching works, or what driving a CLI without a tool pack (such as GoAlign) gets you |
| [Building from source](building-from-source.md) | You are compiling the app yourself or adding a command |
| [Troubleshooting](troubleshooting.md) | Something failed and the message is not obvious |

Chinese documentation: [`docs/zh/`](../zh/README.md).

## The one-minute model

PhyloPanel is a graphical driver for a phylogenetics command-line tool. It has four moving
parts:

1. **Reflection.** On startup it runs `<tool> --help` for every command it can discover and
   builds the command tree and the flag forms from that output. The GUI is a rendering of
   the CLI, never a re-implementation of it.
2. **Input.** A tree arrives either as a file path (passed with `--input`) or as pasted text
   (written to the first process's stdin).
3. **Workflow.** An ordered list of commands. Step *n*'s stdout is piped into step *n+1*'s
   stdin by the operating system.
4. **Result.** The last step's stdout, rendered as text, an SVG drawing or a PNG image.

Everything else in the interface serves one of those four.

## Conventions used in these pages

- `monospace` — something you type, or text exactly as it appears in the UI.
- **Bold** — a control you click or a field you fill.
- "GoTree" is the bundled command-line tool; "the panel" is PhyloPanel itself.
