# rbw-pinentry

This is a pinentry program for [`rbw`](https://github.com/doy/rbw/tree/main) that get the master password from your keyring. It works the same as [`rbw-pinentry-keyring`](https://github.com/doy/rbw/blob/main/bin/rbw-pinentry-keyring) but with a bit more functionality like notifications in case of a problem.

Maybe this program will evolve to be used with more apps in the future.

## Configuration

In your `rbw` configuration you must point the key `pinentry` to this executable.

## Usage

Use `rbw-pinentry store` to set up the master password in your keyring.
