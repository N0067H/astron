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
integer/float remainder, fail in the interpreter and remain unsupported in native programs.

Native compilation also supports `scan`/`fallback`, `route`, `burn`,
`spin` over exclusive or inclusive integer ranges, and `orbit`. `eject` and
`pass` break and continue loops. Stages support parameters, return values,
forward calls, and recursion. A stage's variables are isolated from its caller.
`ignite` runs before the selected launch, and its variables remain available
in that launch. `abort` ends a launch with code 1; inside a stage it ends that
call with no value, matching the interpreter.

String literals, string stage parameters and returns, equality, and `log` are
supported by the control-flow lowering path so existing examples 01, 02, 04,
05, 06, 08, 09, and 10 can build. Arrays, enums, and other built-ins remain
unsupported. Native runtime error diagnostics, full floating-point log
formatting compatibility, and integer overflow behavior remain future work.
Use nonzero divisors and in-range integer arithmetic for now.

Simple scalar programs generate textual LLVM IR directly. Programs using
control flow, stages, or ignite first lower to C11 with tagged values that
preserve runtime scalar types, then Clang generates LLVM IR. Both paths invoke
`clang -x ir -c` to produce an object file and Clang again to link it.
Intermediate files are removed after success or failure. Clang must be on
`PATH`; build errors identify the failing compilation or linking phase.
