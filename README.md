# aspl - A Simple Programming Language

**aspl** is a simple, intuitive programming language inspired by the Bash shell and Unix command-like systems.

## Language

### Basic

```bash
set msg "The apple count is: "
set count 10
logl msg count # The apple count is: 10
```

### Data Types

```bash
set int 64
set str "Hello, World"
set bool true
set arr [1 2 3 4]
```

### Math

```bash
set num1 32
set num2 32
set result @math(num1 + num2)

logl result # 64
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

set msg @get
@print msg
```

### Sourcing

```bash
# utils.aspl
fn util_foo {
  logl "foo bar"
}
```

```bash
# main.aspl
@source "./utils.aspl"

@util_foo
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

## Contribution

Contributions to aspl are welcome! If you have ideas for improvements, new features, or bug fixes, feel free to open an issue or submit a pull request on [aspl](https://github.com/qxb3/aspl)

## LICENSE

[LICENSE](https://github.com/qxb3/eiv/blob/main/LICENSE)
