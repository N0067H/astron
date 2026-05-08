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
| null    | `air`    |
| array   | `[type]` |
| enum    | `enum`   |

```astrn
payload nums: [int] = [1, 2, 3]
fuel names: [str] = ["apollo", "artemis"]
payload checksum: byte = 0xFF
fuel active: flag = true
fuel nothing: str = air
```

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

## Operators

**Arithmetic:** `+` `-` `*` `/` `%`

**Comparison:** `==` `!=` `<` `>` `<=` `>=`

**Assignment:** `=` `+=` `-=` `*=` `/=` `%=`

**Logical:** `and` `or` `not`

**Range:** `..` (exclusive), `..=` (inclusive)

```astrn
scan x > 0 and y > 0 { }
scan x == 0 or y == 0 { }
scan not flag { }
```
