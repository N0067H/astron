# Getting Started

## Installation

```sh
git clone https://github.com/N0067H/astron
cd astron
cargo build --release
```

The binary is at `target/release/astron` (`target/release/astron.exe` on Windows).
Add its directory to `PATH`, or install the CLI with `cargo install --path .`.
Running the interpreter does not require Clang.

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

The default entry point is `launch main`. `land <int>` ends a launch with that
exit code. Reaching the end of a launch, or returning `air`, exits with code 0.

## Building a Native Executable

Install Clang and a working system linker, then run:

```sh
astron build exam/13_builtin_functions.astrn
astron build exam/13_builtin_functions.astrn --launch main -o ./mission
./mission
```

On Windows, use `-o mission.exe` and run `.\mission.exe`. Clang and the linker
are needed when building; running the resulting executable does not invoke
Astron or Clang. The executable may still depend on system runtime libraries.

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
integer/float remainder, report the same runtime errors in interpreted and
native programs.

Native compilation also supports `scan`/`fallback`, `route`, `burn`,
`spin` over exclusive or inclusive integer ranges, and `orbit`. `eject` and
`pass` break and continue loops. Stages support parameters, return values,
forward calls, and recursion. A stage's variables are isolated from its caller.
`ignite` runs before the selected launch, and its variables remain available
in that launch. `abort` ends a launch with code 1; inside a stage it ends that
call with no value, matching the interpreter.

Native compilation supports strings, arrays, enums, and all four built-ins:
`log`, `len`, `assert`, and `to_str`. Strings support concatenation, comparisons,
and Unicode scalar value lengths. Arrays support literals, indexing, element
assignment, compound assignment, and value copies across variables and stage
calls. Enum values support comparisons, route matching, and display as
`Enum.Variant`. Arrays display recursively, including nested arrays.

Runtime errors report the source path, line, and column and exit with code 1.
This includes division and remainder by zero, invalid array indices, failed
assertions, invalid runtime value types, and inclusive range end overflow.

Integer addition, subtraction, multiplication, and negation wrap at 64 bits in
both execution modes. Division or remainder of the minimum integer by `-1`
reports an overflow error. `air` is a void value: it displays as an empty string,
compares equal to another void value, and fails when used as a number or flag.
A stage that aborts or returns without a value produces `air`. Launch and ignite
`abort` exit with code 1; a non-integer launch return exits with code 0.

Supported scalar operations generate textual LLVM IR directly, with runtime
checks before division and remainder. Other expressions use the C11 runtime.
Programs using strings, arrays, enums, built-ins, control flow, stages, or ignite first lower
to C11 with tagged values that preserve runtime value types, then Clang
generates LLVM IR. Both paths invoke
`clang -x ir -c` to produce an object file and Clang again to link it.
Intermediate files are removed after success or failure. Clang must be on
`PATH`; build errors identify the failing compilation or linking phase.

Native builds use the host platform; `build` has no cross-compilation option.
Native runtime allocations are retained until process exit, so long-running
programs that repeatedly create strings or arrays can use increasing memory.
For error messages and overflow rules, see [Runtime Errors](./runtime-errors.md).
