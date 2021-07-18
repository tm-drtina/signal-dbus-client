# signal-dbus-client
D-Bus based client for Signal Messenger

## Binaries
### Register
Provide device name and optionally file, where credentials will be stored. If no credentials file is specified, credentials will be printed to stdout (safe to pipe to file).

### Client
Send one message to recipient.

### Daemon
TBD

## Issues
- Registration overrides previous device

## Development
### Update signal certificate
`openssl s_client -connect textsecure-service.whispersystems.org:443 -showcerts </dev/null | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > signal_certs.pem`
