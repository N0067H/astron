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

The LLVM backend supports `int`, `float`, `flag`, and `byte` literals, local
`payload` and `fuel` variables, assignments (including compound assignments),
arithmetic, comparisons, unary negation, and short-circuit `and`/`or`/`not`.
For example, `exam/14_scalar_calculations.astrn` builds and exits with code 42.
`land` accepts integer expressions, with the result truncated to a 32-bit
process exit code (the operating system may further restrict its range).

Mixed `int`/`float` arithmetic and comparisons follow the interpreter. A
variable's actual value type is preserved: initializing a `float` variable with
an integer does not itself convert the value to floating point. `byte` supports
storage, assignment, and equality; byte arithmetic and ordering, and mixed
integer/float remainder, fail in the interpreter and are rejected during native
compilation.

Nonempty `ignite` blocks, control-flow statements, stage calls, strings, arrays,
and built-ins still report a source-located build error. The Hello World
example above still requires the interpreter. Native runtime error checks
(including division by zero) and integer overflow behavior will be addressed
in the runtime compatibility work; use nonzero divisors and in-range integer
arithmetic for now.

The build generates textual LLVM IR in a private temporary directory, invokes
`clang -x ir -c` to produce an object file, then invokes `clang` again to link
it. Intermediate files are removed after success or failure. Clang must be on
`PATH`; build errors identify the failing compilation or linking phase.
