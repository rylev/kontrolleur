# kontrolleur

Inpsect a wasm binary file to see what it expects from its environment.

For instance, kontrolleur can tell whether a wasm binary needs a wasi compliant runtime and if so, it can give insight into what types of system resources the binary is likely to use.

## Use

```
USAGE:
    kontrolleur [FLAGS] <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Verbose output

ARGS:
    <file>    Input file
```
