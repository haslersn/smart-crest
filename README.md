# smart_crest

Connects to a PC/SC smart card reader and posts any read ATR string to a
configured HTTP endpoint.

## Build

```bash
$ cargo build
```

### With Nix

```bash
$ nix-build
```

## Configuration

In the working directory where smart_crest is executed, there must be a
`smart_crest.toml` configuration file.
A good start is to copy the `smart_crest.toml.example` from this repository.

### top-level keys

#### `endpoint =`

The HTTP endpoint to send a `POST` request to, any time a smart card is
detected and its ATR string is successfully read.
In the endpoint URL, any instance of `{}` is replaced by the ATR string.
Example: `"http://localhost:8010/entman/access?token={}"`

NOTE:
The read ATR string has its last two bytes chopped off, for an undocumented
reason.
