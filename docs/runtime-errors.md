# Runtime Errors

Astron reports runtime failures with the source file, line, and column.

```text
path/to/file.astrn:line:col: runtime error: message
```

These errors happen after parsing and type checking, while the program is running.

## Errors Before Execution

The interpreter and `build` command both parse and type-check the program
before execution or native code generation. Undefined names, invalid argument
counts, incompatible declared types, and ranges outside `spin` are normally
rejected at this stage. Those diagnostics do not include `runtime error:`.
For example, `fire unknown_stage()` is a type-checking error, not a runtime
failure.

A missing launch is reported differently by the two CLI modes. Interpreting
with `--launch missing` reports:

```text
path/to/file.astrn:1:1: runtime error: no 'launch missing' found
```

Building with the same option reports `build error: no 'launch missing' found`
and does not produce an executable.

## Array Index Errors

Array indexes must be non-negative integers and must stay within bounds.

```astrn
launch main {
    payload values: [int] = [10, 20]
    fire log(values[5])
    land 0
}
```

```text
example.astrn:3:21: runtime error: array index 5 out of bounds with length 2
```

```astrn
launch main {
    payload values: [int] = [10, 20]
    fire log(values[-1])
    land 0
}
```

```text
example.astrn:3:21: runtime error: array index must be non-negative, got -1
```

Index assignment reports the array name in the message.

```text
example.astrn:3:12: runtime error: array index 5 out of bounds for 'values' with length 2
```

A variable declared as an array can contain `air`. Indexing that value fails
at runtime; directly indexing a string is rejected by the type checker.

```text
example.astrn:3:14: runtime error: cannot index into void
```

## Condition Errors

Runtime conditions must evaluate to `flag`.

This applies to:

- `scan`
- `burn`
- `and`
- `or`
- `not`
- `assert`

A variable declared as `flag` can contain `air`. If that void value reaches
one of those operations, Astron reports:

```text
example.astrn:3:10: runtime error: expected flag, got void
```

## Arithmetic Errors

Astron reports invalid arithmetic at runtime when the operation is not defined for the values it receives.

```astrn
launch main {
    fire log(0x01 + 0x02)
    land 0
}
```

```text
example.astrn:2:14: runtime error: operator '+' is not defined for byte and byte
```

Division and modulo by zero are also reported explicitly.

```astrn
launch main {
    payload amount: int = 10
    fire log(amount / 0)
    land 0
}
```

```text
example.astrn:3:14: runtime error: division by zero
```

```text
example.astrn:3:14: runtime error: modulo by zero
```

A numeric variable containing `air` fails when negated.

```text
example.astrn:3:14: runtime error: cannot negate void
```

## Integer Overflow

Integers are signed 64-bit values. Addition, subtraction, multiplication, and
negation wrap modulo 2^64 in both interpreted and native programs, regardless
of whether Astron itself was built in debug or release mode.

For example, `9223372036854775807 + 1` produces `-9223372036854775808`.
Negating the minimum integer produces the same minimum integer.

Dividing the minimum integer by `-1` cannot produce a signed 64-bit result.
Division and remainder report errors instead of panicking or invoking native
undefined behavior:

```text
example.astrn:3:14: runtime error: integer division overflow
example.astrn:3:14: runtime error: integer remainder overflow
```

## Comparison Errors

Some values cannot be compared with `<`, `>`, `<=`, or `>=`.

```text
example.astrn:3:10: runtime error: cannot compare byte and byte
```

## Range Errors

Range expressions are only valid inside `spin`; the type checker rejects
other uses. A bound declared as `int` but containing `air` fails at runtime:

```text
example.astrn:3:18: runtime error: expected int, got void
```

Inclusive ranges that overflow the end bound also fail.

```text
example.astrn:3:10: runtime error: inclusive range end overflowed
```

## Built-in Function Errors

Built-in argument counts and declared types are checked before execution.
Runtime checks still apply to values that are declared with an accepted type
but actually contain `air`, for example:

```astrn
launch main {
    payload name: str = air
    fire log(fire len(name))
    land 0
}
```

The call reports `'len' expects array or str, got void`. Likewise, passing a
`flag` variable containing `air` to `assert` reports `expected flag, got void`.

If `assert` receives `false`, Astron reports:

```text
example.astrn:3:10: runtime error: assertion failed
```

## Air and Abort

`air` is a void value, not a numeric zero or a false flag. It displays as an
empty string and compares equal to another void value. Using it in arithmetic,
a condition, an index, or a range bound produces the corresponding runtime
type error, even if its variable has a numeric or flag declaration.

A stage produces `air` when it reaches its end, executes `abort`, or returns
without a value. Stage-local `abort` returns control to the caller. In ignite
or a launch, `abort` exits the program with code 1 without an error message.

An integer `land` value supplies the exit code through a signed 32-bit
conversion; operating systems may further restrict it, such as to the low
8 bits on Unix. Launch return expressions are checked as `int`, but `air`
is allowed and exits with code 0. In `ignite`, a bare `land` or `land air`
continues execution at the selected launch.

Native diagnostics embed the source paths and locations from compilation,
including imported modules. Short-circuited expressions do not raise errors
or evaluate their right-hand operands.

## Notes

- Many invalid programs are rejected earlier by the parser or type checker.
- Runtime errors only cover failures that happen during execution.
