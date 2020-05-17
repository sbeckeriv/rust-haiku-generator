# rust-haiku-generator
rust haiku generator using markov chains. 
```
USAGE:
    haiku-generator [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Sets the input file to use

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --stored <FILE>    Use yaml files instead of input. Will generate the file on first run.
```

markov code from https://docs.rs/markov/1.0.3/markov/ . I needed access to the order and map which are private.
