# 🚀 The Astron Programming Language

<center>
<img src="./logo.png" width=300px>

### Launch fast. Land clean.

Just **ENJOY** it;
</center>

Astron is a fun programming language where every keyword is borrowed from the world of rockets and space exploration.

## Installation

Build from source:

```sh
cargo build --release
```

Or install the CLI directly with Cargo:

```sh
cargo install --path .
```

## CLI

```sh
astron <file.astrn>
astron <file.astrn> --launch <name>
astron build <file.astrn> --launch <name> -o <executable>
astron --help
astron --version
```

The first two commands run the interpreter. `build` creates a standalone native
executable and requires Clang and a working system linker on `PATH`.
`--launch` defaults to `main` in both modes.

```sh
astron build exam/13_builtin_functions.astrn -o ./mission
./mission
```

On Windows, use `-o mission.exe` and run `.\mission.exe`.
See [Getting Started](./docs/getting-started.md#building-a-native-executable)
for supported features, exit codes, and runtime requirements.

## Documentation

- [Getting Started](./docs/getting-started.md)
- [Syntax Reference](./docs/syntax.md)
- [Built-in Functions](./docs/builtins.md)
- [Runtime Errors](./docs/runtime-errors.md)

---

Check the <a href="./exam/">example code</a>!
