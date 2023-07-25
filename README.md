# Price Notification Cli

Track coins from coingecko and receive sms notification at desired price points

## Install

`cargo install price-notifications`

## How to use

To receive notifications you must setup an account on [Twilio](https://www.twilio.com/)


## Example

First time executing the cli will open up dialoguer to setup twilio details and will download a list of supported currencies to `home/user/.config/price-notifications`

```bash
❯ price-notifications

Log is written to /home/user/.config/price-notifications/logs/logs_2023-07-25_09-59-55.log

Creating new config file
creating dir: "/home/user/.config/price-notifications"

Twilio number from: +111111111111111
Twilio number to: +12222222222222
Twilio account id: yourtwilioaccountid
Twilio auth token: [hidden]
Successfully created new config file!

```

```bash
❯ price-notifications

Log is written to /home/user/.config/price-notifications/logs/logs_2023-07-25_09-55-07.log

❯ Get prices
  Add Currency
  Remove Currency
  Add Notification
  Update supported currencies
  Show config
```
