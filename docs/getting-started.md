# Getting Started

## Installation

```sh
git clone https://github.com/SeungYeop/astron
cd astron
cargo build --release
```

The binary is at `target/release/astron`.

## Running a Program

```sh
astron <file.astrn>
astron <file.astrn> --launch <name>
astron --help
astron --version
```

`--launch` selects which `launch` block to run. Defaults to `main`.

## Hello World

```astrn
launch main {
    fire log("hello, world")
    land 0
}
```

Every `launch` block must end with `land <int>`. The default entry point is `launch main`.

## Building a Native Executable

Install Clang and a working system linker, then run:

```sh
astron build minimal.astrn
astron build minimal.astrn --launch main -o ./minimal-app
./minimal-app
```

The default executable path is the input path with `.astrn` removed (`.exe`
replaces it on Windows). `--launch` defaults to `main`. Existing interpreter
commands remain available.

The initial LLVM backend supports a minimal launch block such as
`launch main { land 0 }`. The integer literal becomes the process exit code.
Nonempty `ignite` blocks and other executed syntax report a source-located
build error until the expression, control-flow, and runtime backends are added.
The Hello World example above still requires the interpreter.

The build generates textual LLVM IR in a private temporary directory, invokes
`clang -x ir -c` to produce an object file, then invokes `clang` again to link
it. Intermediate files are removed after success or failure. Clang must be on
`PATH`; build errors identify the failing compilation or linking phase.
