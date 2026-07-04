# Built-in Functions

## `log`

Prints values to stdout, separated by spaces.

```astrn
fire log("hello")
fire log(x, y, z)
```

Accepts any number of arguments of any type. Returns `void`.

## `len`

Returns the length of a string or array.

```astrn
payload name: str = "astron"
payload nums: [int] = [1, 2, 3]

fire log(fire len(name))
fire log(fire len(nums))
```

Accepts exactly one argument. Returns `int`.

## `assert`

Aborts the program if the condition is false.

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
