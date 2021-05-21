# ooofa - 2fa in console

ooofa expects a `.ooofa.yaml` file in your HOME directory.

The file contains a maps of keys indexed by a friendly name:

```
keys:
  name: otpauth://totp/issuer:account?secret=AAAABBBB
  name2: otpauth://totp/issuer2:account2?secret=AAAACCCCDDDD
```

then to run:

```
$ ooofa name
213123
(23.3s)
```

or:

```
$ ooofa watch
```
