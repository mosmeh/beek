# beek

[![build](https://github.com/mosmeh/beek/workflows/build/badge.svg)](https://github.com/mosmeh/beek/actions)

A modern CLI calculator

## Web version

[Web version](https://mosmeh.github.io/beek/)

## Installation

Clone this repository and run:

```sh
cargo install --path .
```

## Reference

### Operators (from highest precedence to lowest)

| Category                  | Operators               |
|---------------------------|-------------------------|
| factorial                 | `!`                     |
| exponentiation            | `^`, `**`               |
| implicit multiplication   | whitespace              |
| modulo                    | `%`                     |
| multiplication / division | `*`, `·`, `×`, `/`, `÷` |
| addition / subtraction    | `+`, `-`                |

### Commands

| Command                          | Description           |
|----------------------------------|-----------------------|
| `help`, `?`                      | show help             |
| `list`, `ls`, `ll`, `dir`        | list variables        |
| `delete`, `del`, `rm` *variable* | delete variable       |
| `reset`                          | reset environment     |
| `clear`                          | clear screen          |
| `quit`, `exit`                   | quit                  |

### Variable assignment

There are two types of assignments.

- __Immediate Assignment__ (`:=`) : values are evaluated at declaration time

```text
> mass := 5; velocity := 3
> energy := mass * velocity^2 / 2
 = (5 × (3 ^ 2)) / 2 = 22.5

> mass := 3
> energy
 = 22.5
```

- __Lazy Assignment__ (`=`) : values are evaluated when they are used

```text
> mass = 5; velocity = 3
> energy = mass * velocity^2 / 2
 = (5 × (3 ^ 2)) / 2 = 22.5

> mass = 3
> energy
 = (mass × (velocity ^ 2)) / 2 = ... = 13.5
```

Undefined variables are left unevaluated:

```text
> f = 5x + 1
 = (5 × x) + 1
> f + 3
 = ((5 × x) + 1) + 3

> x = 2
> f + 3
 = ((5 × x) + 1) + 3 = ... = 14
```

## Development

beek's web version is made with WebAssembly. The following commands will build the Rust codes, convert them to WebAssembly, and opens the web version in the browser.

```sh
npm install
npm start
```

To build the web version with optimizations enabled, run:

```sh
npm run build
```
