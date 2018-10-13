BitBot.rs
=========

A simple telegram bot that reads from a toml configuration file then sends input from stdin
to specified telegram account.

```sh
$ echo "Hello, Telegram!" | bb
```

Installation
------------
Run
```sh
$ make install
```

This will install `bb` to your `~/.local/bin` folder.

You will need to ensure that your local bin folder is in your `PATH`.
