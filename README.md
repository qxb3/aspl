# aspl - A Shitty Programming Language

A really shitty bash, command like programming language.

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
  logl "a is greater than b"
}
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
