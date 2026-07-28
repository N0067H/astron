# Runtime Errors

Astron reports runtime failures with the source file, line, and column.

```text
path/to/file.astrn:line:col: runtime error: message
```

These errors happen after parsing and type checking, while the program is running.

## Undefined Names

Accessing a variable that does not exist reports an error.

```astrn
launch main {
    fire log(missing_value)
    land 0
}
```

```text
example.astrn:2:14: runtime error: undefined variable 'missing_value'
```

Calling a stage that does not exist also reports an error.

```astrn
launch main {
    fire unknown_stage()
    land 0
}
```

```text
example.astrn:2:5: runtime error: undefined function 'unknown_stage'
```

If the requested launch block does not exist, Astron exits with an error.

```text
path/to/file.astrn:1:1: runtime error: no 'launch diagnostics' found
```

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

Trying to index into a non-array value also fails at runtime.

```text
example.astrn:3:14: runtime error: cannot index into str
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

If a non-`flag` value reaches one of those operations, Astron reports:

```text
example.astrn:3:10: runtime error: expected flag, got int
```

## Arithmetic Errors

Astron reports invalid arithmetic at runtime when the operation is not defined for the values it receives.

```astrn
launch main {
    fire log("fuel" + 1)
    land 0
}
```

```text
example.astrn:2:14: runtime error: operator '+' is not defined for str and int
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

Negating a non-numeric value also fails.

```text
example.astrn:3:14: runtime error: cannot negate str
```

## Comparison Errors

Some values cannot be compared with `<`, `>`, `<=`, or `>=`.

```text
example.astrn:3:10: runtime error: cannot compare array and str
```

## Range Errors

Range expressions are only valid inside `spin`.

```text
example.astrn:3:18: runtime error: range expression is only allowed in spin
```

If a `spin` loop receives an invalid range during execution, Astron reports it.

```text
example.astrn:3:10: runtime error: spin requires a range expression
```

Inclusive ranges that overflow the end bound also fail.

```text
example.astrn:3:10: runtime error: inclusive range end overflowed
```

## Built-in Function Errors

Built-in stages validate argument counts and runtime value kinds.

```text
example.astrn:3:14: runtime error: 'len' expects 1 argument, got 2
example.astrn:3:14: runtime error: 'len' expects array or str, got flag
example.astrn:3:14: runtime error: 'assert' expects 1 argument, got 0
example.astrn:3:14: runtime error: 'to_str' expects 1 argument, got 2
```

If `assert` receives `false`, Astron reports:

```text
example.astrn:3:10: runtime error: assertion failed
```

## Stage Call Errors

User-defined stages validate argument counts at runtime.

```text
example.astrn:4:10: runtime error: function 'boost' expects 2 arguments, got 1
```

## Notes

- Many invalid programs are rejected earlier by the parser or type checker.
- Runtime errors only cover failures that happen during execution.
