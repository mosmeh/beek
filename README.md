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

### Operators

| Operators                   | Description    |
|-----------------------------|----------------|
| `+`                         | addition       |
| `-`                         | subtraction    |
| `*`, `·`, `×`, *whitespace* | multiplication |
| `/`, `÷`                    | division       |
| `%`                         | modulo         |
| `^`, `**`                   | exponentiation |
| `!`                         | factorial      |

Precedence and associativity (ordered from highest precedence to lowest):

| Category       | Operators                    | Associativity |
|----------------|------------------------------|---------------|
| factorial      | `!`                          | left          |
| exponentiation | `^`, `**`                    | right         |
| multiplication | *whitespace*                 | left          |
| multiplication | `*`, `·`, `×`, `/`, `÷`, `%` | left          |
| addition       | `+`, `-`                     | left          |

### Built-in functions

`abs`, `acos`, `acosh`, `asin`, `asinh`, `atan`, `atan2`, `atanh`, `cbrt`, `ceil`, `cos`, `cosh`, `degrees`, `erf`, `erfc`, `exp`, `floor`, `fract`, `gamma`, `hypot`, `lgamma`, `ln`, `log`, `log10`, `log2`, `max`, `min`, `pow`, `radians`, `random`, `round`, `sign`, `sin`, `sinh`, `sqrt`, `tan`, `tanh`, `trunc`

### Commands

| Command                                         | Description                                          |
|-------------------------------------------------|------------------------------------------------------|
| `help`, `?`                                     | show help                                            |
| `list`, `ls`, `ll`                              | list constants, variables and user-defined functions |
| `delete`, `del`, `rm` *variable*/*function* ... | delete variable(s) or function(s)                    |
| `reset`                                         | reset environment                                    |
| `clear`, `cls`                                  | clear screen                                         |
| `quit`, `exit`                                  | quit                                                 |

### Variable assignment

```
> mass = 5; velocity = 3
> energy = mass * velocity^2 / 2
 = 22.5
```

### Function definition

```
> binomial(n, k) = n! / k! / (n-k)!
> binomial(5, 3)
 = 10
```

### Math and special constants

- `e`, `π` (`pi`) and `τ` (`tau`)
- `ans` and `_` store the last result

## Development

`beek`'s web version is made with WebAssembly. The following commands will build the Rust codes, convert them to WebAssembly, and opens the web version in the browser.

```sh
npm install
npm start
```

To build the web version with optimizations enabled, run:

```sh
npm run build
```
