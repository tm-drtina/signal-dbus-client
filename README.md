# signal-dbus-client
D-Bus based client for Signal Messenger.

**Warning: Currently in very experimental state. Not recommended for use**

## Development
### Update signal certificate
`openssl s_client -connect textsecure-service.whispersystems.org:443 -showcerts </dev/null | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > signal_certs.pem`
