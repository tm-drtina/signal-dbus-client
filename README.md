# signal-dbus-client
D-Bus based client for Signal Messenger

## Issues
- We're not sending sync messages after sending data message
- Resolve "Note to self" type of messages

## Development
### Update signal certificate
`openssl s_client -connect textsecure-service.whispersystems.org:443 -showcerts </dev/null | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > signal_certs.pem`
