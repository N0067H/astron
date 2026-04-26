# 🚀 The Astron Programming Language

<center>
<img src="./logo.png" width=300px>

### Launch fast. Land clean.

Just **ENJOY** it;
</center>

Astron is a fun programming language where every keyword is borrowed from the world of rockets and space exploration.

## 1. Keywords

### 1.1. Entry point

| General     | Astron     |
| ----------- | ---------- |
| main        | **launch** |
| initializer | **ignite** |

```astrn
ignite {
    // global initialization
}

launch main {
}
```

### 1.2. Variables

| General   | Astron      |
| --------- | ----------- |
| immutable | **payload** |
| mutable   | **fuel**    |

```astrn
payload x: int = 10
fuel y: float = 3.14
```

### 1.3. Functions

| General       | Astron      |
| ------------- | ----------- |
| function      | **stage**   |
| call          | **fire**    |
| return type   | **lands**   |

```astrn
stage add(a: int, b: int) lands int {
    land a + b
}

stage greet(name: str) lands void {
    fire log(name)
}

fire add(1, 2)
```

### 1.4. Conditions

| General | Astron       |
| ------- | ------------ |
| if      | **scan**     |
| else    | **fallback** |
| switch  | **route**    |

```astrn
scan x > 10 {
    // something
} fallback {
    // something
}
```

### 1.5. Loops

| General | Astron    |
| ------- | --------- |
| loop    | **orbit** |
| for     | **spin**  |
| while   | **burn**  |

```astrn
orbit {
}

spin i in 0..10 {

}

burn x < 10 {
}
```

### 1.6. Flow controls

| General  | Astron        |
| -------- | ------------- |
| break    | **eject**     |
| continue | **pass**      |
| return   | **land**      |
| exit     | **abort**     |

```astrn
land value
abort
```

### 1.7. Types

| General | Astron    |
| ------- | --------- |
| int     | **int**   |
| float   | **float** |
| string  | **str**   |
| bool    | **flag**  |
| byte    | **byte**  |
| void    | **void**  |
| null    | **air**   |
| array   | **[type]**|

```astrn
payload nums: [int] = [1, 2, 3]
fuel names: [str] = ["apollo", "artemis"]

nums[0]
```

### 1.8. Operators

**Arithmetic**

| Operator | Description    |
| -------- | -------------- |
| `+`      | addition       |
| `-`      | subtraction    |
| `*`      | multiplication |
| `/`      | division       |
| `%`      | modulo         |

**Comparison**

| Operator | Description           |
| -------- | --------------------- |
| `==`     | equal                 |
| `!=`     | not equal             |
| `<`      | less than             |
| `>`      | greater than          |
| `<=`     | less than or equal    |
| `>=`     | greater than or equal |

**Assignment**

| Operator | Description |
| -------- | ----------- |
| `=`      | assign      |
| `+=`     | add assign  |
| `-=`     | sub assign  |
| `*=`     | mul assign  |
| `/=`     | div assign  |
| `%=`     | mod assign  |

**Logical**

| Operator | Description |
| -------- | ----------- |
| `and`    | logical AND |
| `or`     | logical OR  |
| `not`    | logical NOT |

```astrn
scan x > 0 and y > 0 { }
scan x == 0 or y == 0 { }
scan not flag { }
```

**Range**

| Operator | Description              |
| -------- | ------------------------ |
| `..`     | exclusive (`0..10` → 0–9)|
| `..=`    | inclusive (`0..=10` → 0–10)|

```astrn
spin i in 0..10 { }     // 0, 1, 2 ... 9
spin i in 0..=10 { }    // 0, 1, 2 ... 10
```
---

Check the <a href="./exam/mission.astrn">example code</a>!