# Built-in Functions

All four built-ins are available in the interpreter and native executables.
Arguments are evaluated left to right before the call.

## `log`

Prints values to stdout, separated by spaces, followed by a newline.

```astrn
fire log("hello")
fire log(x, y, z)
```

Accepts any number of arguments of any type. `fire log()` prints an empty line.
Returns `void`, whose runtime value is `air`. See [Value Formatting](#value-formatting)
for how each type is displayed.

## `len`

Returns the number of elements in an array or Unicode scalar values in a string.
String lengths count neither UTF-8 bytes nor visual characters: a base letter
and a combining accent count separately. An empty string or array has length 0.

```astrn
payload name: str = "astron"
payload nums: [int] = [1, 2, 3]

fire log(fire len(name))
fire log(fire len(nums))
```

Accepts exactly one argument. Returns `int`.

## `assert`

Reports `assertion failed` on stderr and exits with code 1 if the condition is
false. The diagnostic includes the call's source file, line, and column.
This also terminates the program when called inside a stage; stage-local
`abort`, in contrast, returns to its caller.

```astrn
fire assert(temperature < 100)
```

Accepts exactly one `flag` argument. Returns `void`.

## `to_str`

Converts a value to its string representation.

```astrn
payload x: int = 42
fire log(fire to_str(x))
```

Accepts exactly one argument of any type. Returns `str`.

## Value Formatting

`log` and `to_str` use the same value representation.

| Value | Representation |
| ----- | -------------- |
| `int` | Signed decimal, such as `42` or `-7` |
| `float` | Decimal; integral values retain `.0`, such as `3.0` |
| `flag` | `true` or `false` |
| `byte` | Two uppercase hexadecimal digits prefixed with `0x`, such as `0xAF` |
| `str` | The string contents without quotes |
| Enum variant | `Enum.Variant`, such as `Phase.Ready` |
| Array | Elements separated by `, ` inside brackets, such as `[1, 2]` |
| `air` | An empty string |

Nested arrays are formatted recursively. String elements are not quoted, so
an array of strings such as `["nav", "engine"]` displays as `[nav, engine]`.
Formatting is for display; it is not a serialization format.

Argument count and declared type errors are rejected before execution. Runtime
value checks still apply when a typed variable actually contains `air`.
See [Runtime Errors](./runtime-errors.md).
