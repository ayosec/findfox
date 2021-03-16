# findfox

Send remote commands to a Firefox instance.

By default, a command like `firefox https://example.com` opens the URL in the
first window. If there are multiple Firefox instances there is no way to select
which instance receives the URL. This is important if there are multiple
Firefox instances running in containers.

`findfox` is able to locate all Firefox instances that can receive a remote
command, and send any command to a specific one.

## Usage

To get all available instances, just type `findfox`:

```console
$ findfox
c1bf597b6e83 83886
06487346c798 106015
e1ca8fca6866 23068
```

The first column is the hostname where the instance is running (in this
example, the container name), and the second one is the window identifier.

Then, to send a URL:

```bash
findfox send $WINDOW args*
```

`args*` is the command line required to send the command. For example:

```console
$ findfox send 10615 firefox --private-window https://example.com
```

## Compatibility

The implementation is tested against Firefox 85 on X11.
