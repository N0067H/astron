# Syntax Reference

## Entry Point

| General     | Astron     |
| ----------- | ---------- |
| main        | `launch`   |
| initializer | `ignite`   |

```astrn
ignite {
    // runs before launch main
}

launch main {
    land 0
}
```

> **Note:** Variables declared in `ignite` are currently not accessible inside `launch main`. See [known issues](./known-issues.md).

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

```astrn
payload nums: [int] = [1, 2, 3]
fuel names: [str] = ["apollo", "artemis"]
payload checksum: byte = 0xFF
fuel active: flag = true
fuel nothing: str = air
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
