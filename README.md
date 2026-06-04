# Belt

Belt is the official build system and project manager for the Unnameable programming language.

## Installation

Build from source:

```bash
cargo build --release
```

Then symlink the binary to your PATH:

```bash
ln -s /path/to/belt/target/release/belt ~/.local/bin/belt
```

## Quick Start

Create a new project:

```bash
belt new myproject
cd myproject
belt build
belt run
```

## Commands

| Command | Description |
|---|---|
| `belt new <name>` | Create a new project |
| `belt build` | Build the project |
| `belt run` | Build and run the executable |
| `belt check` | Run frontend checks without compiling |
| `belt clean` | Remove build artifacts |
| `belt help` | Show help |
| `belt <command>` | Run a user defined command from belt.lethr |

## Project Structure

Running `belt new myproject` creates:

```
myproject/
├── belt.lethr        # Project configuration
├── src/
│   └── main.unn      # Entry point
├── stubs/            # Generated stub files
├── objs/             # Compiled object files
└── build/            # Final build output
```

## Configuration : belt.lethr

Belt uses a custom configuration format called Leather with the `.lethr` extension.

### [project]

```
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.unn"
mode = executable
```

| Field | Description | Required |
|---|---|---|
| `name` | Project name | Yes |
| `version` | Project version | Yes |
| `entry` | Entry point file | Yes for executable |
| `mode` | `executable`, `static`, or `obj` | Yes |
| `target` | Target triple for cross compilation | No |
| `freestanding` | Disable standard library linking | No |
| `script` | Path to custom linker script | No |

### [layout]

Override default folder locations. All fields are optional. 
Belt uses sensible defaults.

```
[layout]
src = "src"
stubs = "stubs"
objs = "objs"
build = "build"
```

### [dependencies]

Local Unnameable project dependencies.

```
[dependencies]
mylib = "../mylib"
```

### [link]

external libraries to link against.

```
[link]
c = ["pthread", "SDL2"]
```

### [commands]

User defined commands invokable via `belt <name>`.

```
[commands]
qemu = "qemu-system-x86_64 -kernel build/myos.elf -m 256M -serial stdio"
iso = "grub-mkrescue -o build/myos.iso build/iso"
debug = "qemu-system-x86_64 -kernel build/myos.elf -s -S"
```

Then run with:

```bash
belt qemu
belt iso
belt debug
```

## Cross Compilation

Specify a target triple in `belt.lethr`:

```
[project]
name = "myproject"
version = "0.1.0"
entry = "src/main.unn"
mode = executable
target = "x86_64-unknown-linux-gnu"
```

Belt automatically adjusts file extensions based on the target.

## OS / Bare Metal Development

Belt has first class support for freestanding targets:

```
[project]
name = "myos"
version = "0.1.0"
entry = "src/main.unn"
mode = executable
target = "x86_64-unknown-none"
freestanding = true
script = "scripts/kernel.ld"

[commands]
qemu = "qemu-system-x86_64 -kernel build/myos.elf -m 256M -serial stdio -display none"
debug = "qemu-system-x86_64 -kernel build/myos.elf -s -S -serial stdio"
```

## Incremental Builds

Belt automatically tracks file changes via `belt.lock`. 
Only modified files and their dependents are recompiled on subsequent builds.

To force a full rebuild:

```bash
belt clean
belt build
```

## Module System

Unnameable files declare their module name at the top:

```
module mymodule

import anothermodule
```

Belt resolves module names to file paths, generates stubs for dependencies, and passes them to the compiler in the correct order. The compiler never searches for files, Belt handles all resolution.

## Requirements

- [unnc](https://github.com/BananaChristian/Unnameable) — Unnameable compiler
- ld.lld — LLVM linker

## License
MIT

