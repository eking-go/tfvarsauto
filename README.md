# tfvarsauto

Just utility that creates variables definition and outputs definition from terraform code

You need to have rust to build binary

```
cargo build
```

How to install Rust you can read in official Rust documentation ;)

```
$ ./tfvarsauto -h
Usage: tfvarsauto [-h] [-m main.tf] [-v vars.tf] [-o outputs.tf]

Creates variables definition and
outputs definition from terraform code

Options:
    -h, --help          Print the usage menu
    -m, --main main.tf  The name of file for input with main terraform code,
                        main.tf by default
    -v, --vars vars.tf  The name of file to output with variables definition,
                        vars.tf by default
    -o, --outputs outputs.tf
                        The name of file to output with outputs definition,
                        outputs.tf by default
```
