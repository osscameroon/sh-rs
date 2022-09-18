# sh-rs

This is a simple "shell" made live on stream with the community. The replay is 
available [here](https://t.me/c/1385339271/45772)

# Usage

Make sure you have [cargo](https://doc.rust-lang.org/stable/cargo/) installed, clone the repository then:

```shell
cargo run 
```
You have a prompt with the current directory, you can run command and see the output, and that's about it.

# Ideas for extension

## Add Shell Built-ins

The essentials `cd`, `exit` and `help`. If you want more inspiration looks for builtin in `man bash`

## Support for Piping and redirecting

That might require rewriting the basic principle of the shell.
