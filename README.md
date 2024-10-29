# aspl - A Simple Programming Language

**aspl** is a simple, intuitive programming language inspired by the Bash shell and Unix command-like systems.
It is designed to streamline scripting tasks and command execution with a syntax that feels familiar to those accustomed to Unix environments.

## Language

### Basic

```bash
set msg "The apple count is: "
set count 10
logl msg count # The apple count is: 10
```

### Conditional

```bash
set a 10
set b 20

check a < b {
  logl "a is less than b"
}
```

### While loop
```bash
while true {
  logl "loop"
}
```

### Function
```bash
fn get {
  ret "foobar"
}

fn print arg1 {
  logl arg
}

set msg get
print msg
```

## Using

### Installation

```
cargo install aspl
```

### Running

```
aspl <input.aspl>
```

## LICENSE

[LICENSE](https://github.com/qxb3/eiv/blob/main/LICENSE)
