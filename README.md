# signal-dbus-client
D-Bus based client for Signal Messenger

## Issues
- Send message returns HTTP 400

## Development
### Update signal certificate
`openssl s_client -connect textsecure-service.whispersystems.org:443 -showcerts </dev/null | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > signal_certs.pem`
