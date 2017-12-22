# GoDaddy DNS Updater

Update your DNS records on GoDaddy through the GoDaddy Domains API. Register a new key on your [GoDaddy developer page](https://developer.godaddy.com/keys/).

## Usage

`$ godaddy-dns-updater --help`

## `godaddy-dns-updater update`

Updates GoDaddy DNS record with IP address.

### Options

```
-a, --api_key <api_key>          sets the API key for your GoDaddy account
-s, --secret <api_key_secret>    sets the API key secret for your GoDaddy account
-d, --domain <domain>            sets the domain to update DNS records
-i, --ip <ip_address>            sets the IP address to update DNS records
-n, --name <record_name>         sets the name of the record
-l, --ttl <record_ttl>           sets the time to live of the record in seconds [default: 600]
-t, --type <record_type>         sets the type of the record [default: A]
```

### Example - Update DNS record with current IP

`$ godaddy-dns-updater --api_key godaddy_api_key --secret godaddy_api_key_secret --domain example.com --name test`

or,

`$ godaddy-dns-updater -a godaddy_api_key -s godaddy_api_key_secret -d example.com -n test`

### Example - Update DNS record with other IP

`$ godaddy-dns-updater --api_key godaddy_api_key --secret godaddy_api_key_secret --domain example.com --name test --ip 123.123.123.123`

or,

`$ godaddy-dns-updater -a godaddy_api_key -s godaddy_api_key_secret -d example.com -n test -i 123.123.123.123`

## `godaddy-dns-updater cache:clear`

Clears last IP address cache.

### Example

`$ godaddy-dns-updater cache:clear`
