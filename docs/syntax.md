# Syntax Reference

## Entry Point

| General     | Astron     |
| ----------- | ---------- |
| main        | `launch`   |
| initializer | `ignite`   |

```astrn
ignite {
    // runs before any launch — variables declared here are accessible in all launch blocks
    payload version: str = "1.0.0"
}

launch main {
    fire log(version)
    land 0
}
```

Multiple `launch` blocks can be defined in one file. `ignite` runs once before whichever `launch` is selected, and its variables are shared across all of them.

```astrn
ignite {
    payload db_url: str = "localhost:5432"
}

launch main {
    fire log(db_url)
    land 0
}

launch test {
    fire log("running tests against")
    fire log(db_url)
    land 0
}
```

To run a specific launch:

```sh
astron file.astrn --launch test
```

The default is `main` if `--launch` is not specified.

## Imports

Use `import` with a string path to include stages, launches, and ignite declarations from another Astron file.

```astrn
import "./math.astrn"

launch main {
    payload total: int = fire add(2, 3)
    fire log(total)
    land 0
}
```

Import paths are resolved relative to the file that contains the `import`. Each file is loaded once, and cyclic imports are rejected.

## Variables

| General   | Astron    |
| --------- | --------- |
| immutable | `payload` |
| mutable   | `fuel`    |

```astrn
payload x: int = 10
fuel y: float = 3.14
```

All variables require an explicit type annotation.

## Types

| General | Astron   |
| ------- | -------- |
| int     | `int`    |
| float   | `float`  |
| string  | `str`    |
| bool    | `flag`   |
| byte    | `byte`   |
| void    | `void`   |
| void value | `air` |
| array   | `[type]` |
| enum    | `enum`   |

```astrn
payload nums: [int] = [1, 2, 3]
fuel names: [str] = ["apollo", "artemis"]
payload checksum: byte = 0xFF
fuel active: flag = true
fuel nothing: str = air
```

`air` is the runtime value of `void`, not a separate type annotation or a
numeric zero. A typed variable can contain `air`, but numeric and flag
operations check its actual value at runtime.

## Arrays and Value Copies

Arrays use zero-based integer indices. Element mutation requires `fuel`.
Assigning or passing an array copies its value; changing one variable's array
does not change another copy, including copies passed to stages.

```astrn
fuel values: [int] = [1, 2, 3]
payload saved: [int] = values
values[0] += 10
fire log(values, saved)
```

This prints `[11, 2, 3] [1, 2, 3]`. Arrays can contain arrays, and nested arrays
keep the same value-copy behavior. Negative or out-of-bounds indices report
runtime errors. Array `==` is not structural equality: it returns `false`,
even when comparing an array with itself; `!=` returns `true`.

## Enums

Use `enum` to define named variants. Enum variants can be used as values and in `route` patterns.

```astrn
enum Phase {
    Launch
    Orbit
    Landing
}

stage describe(phase: Phase) lands str {
    route phase {
        Launch => land "launch"
        Orbit => land "orbit"
        Landing => land "landing"
    }
    land "unknown"
}

launch main {
    payload phase: Phase = Orbit
    fire log(fire describe(phase))
    land 0
}
```

Variants are referenced by their bare names (`Orbit`, not `Phase.Orbit`).
Variant names must be unique across enums in the loaded program. Local
variables may shadow a variant name. Enum equality compares the enum and
variant; `log` and `to_str` display values as `Phase.Orbit`.

## Functions

| General     | Astron   |
| ----------- | -------- |
| function    | `stage`  |
| call        | `fire`   |
| return type | `lands`  |

```astrn
stage add(a: int, b: int) lands int {
    land a + b
}

stage greet(name: str) lands void {
    fire log(name)
}

fire add(1, 2)
fuel result: int = fire add(1, 2)
```

## Conditions

| General | Astron     |
| ------- | ---------- |
| if      | `scan`     |
| else    | `fallback` |
| switch  | `route`    |

```astrn
scan x > 10 {
    fire log("big")
} fallback {
    fire log("small")
}

route phase {
    1 => fire log("launch")
    2 => fire log("orbit")
    3 => fire log("landing")
}
```

## Loops

| General | Astron  |
| ------- | ------- |
| loop    | `orbit` |
| for     | `spin`  |
| while   | `burn`  |

```astrn
orbit {
    // infinite loop
}

spin i in 0..10 { }     // 0 to 9
spin i in 0..=10 { }    // 0 to 10

burn x < 10 {
    x += 1
}
```

## Flow Control

| General  | Astron   |
| -------- | -------- |
| break    | `eject`  |
| continue | `pass`   |
| return   | `land`   |
| exit     | `abort`  |

```astrn
land value
abort
```

A launch's integer `land` value becomes a signed 32-bit process exit code;
the operating system may restrict it further. A launch may reach its end
without `land`, or use `land air`, to exit with code 0. A bare `land` is valid
in a `void` stage or `ignite`; launch returns are checked as `int`.

A stage produces `air` when it reaches its end, returns `air`, or executes
`abort`. `abort` inside a stage returns to the caller; in `ignite` or a launch,
it ends the process with code 1. `land air` in `ignite` skips the remainder of
that initializer and continues at the selected launch.

## Operators

**Arithmetic:** `+` `-` `*` `/` `%`

**Comparison:** `==` `!=` `<` `>` `<=` `>=`

**Assignment:** `=` `+=` `-=` `*=` `/=` `%=`

**Logical:** `and` `or` `not`

**Range:** `..` (exclusive), `..=` (inclusive)

String `+` concatenates strings. Numeric operations preserve runtime value
types: assigning `7` to a variable declared `float` does not convert it, so
`7 / 2` remains integer division and produces `3`. Mixed `int`/`float`
arithmetic promotes to `float`, except mixed-type `%`, which reports an error.
Bytes support equality but not arithmetic or ordering.

Integers are signed 64-bit values. Addition, subtraction, multiplication, and
negation wrap on overflow. Division and remainder by zero, and division or
remainder of the minimum integer by `-1`, report runtime errors.

`and` and `or` short-circuit. Range bounds are evaluated once, from left to
right; an inclusive end of `9223372036854775807` reports an overflow error.

```astrn
scan x > 0 and y > 0 { }
scan x == 0 or y == 0 { }
scan not flag { }
```
