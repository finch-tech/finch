<div style="text-align:center; margin: 50px 0">
  <img src="docs/finch-logo.png" width="200" />
</div>

## What is Finch

Finch is an open-source cryptocurrency payment processor with a focus on ease of integration and flexibility.

<div style="text-align:center; margin: 50px 0">
    <img src="https://finch.ams3.cdn.digitaloceanspaces.com/branding/finch-modal.gif" width="600" />
</div>

**Note**: Finch is currently in beta form and may yet be subject to significant modification prior to the release of version 1.0.

## Demo

Try a [public demo](https://app.finchtech.io) of Finch and its Management Console.

## Installation

We support two methods of installing and running your own Finch server. Our recommended approach is to use Docker, but if this environment is not supported, you may also set up a Rust environment.

- [Via Docker](https://docs.finchtech.io/docs/installation/installation_with_docker)
- [Via Rust](https://docs.finchtech.io/docs/installation/installation_with_rust)

## Integration with Your Services

Since Finch communicates directly with the client-side of integrated services, our front-end SDK can handle almost everything needed for the integration. We currently provide [JavaScript SDK](https://github.com/finch-tech/finch-sdk-javascript), which allows you to start accepting cryptocurrencies with a block of code;

```js
<script>
  window.onload = function() {
    const finchCheckout = new FinchCheckout({
      apiUrl: "https://api.finchtech.io",
      apiKey: "5tsdghD/RusjgbskoisRrgw==",
      currencies: ["btc", "eth"],
      fiat: "usd",
      price: "1.2",
      identifier: "hello@example.com",
      button: document.getElementById("pay-with-crypto"),
      onSuccess: function(voucher) {
        // Here you can get signed payment vouchers in the form of JSON Web Tokens.
        // On your service’s backend you simply need to verify
        // this voucher using the JWT decode library of your choice.
        console.log("Successfully completed the payment.", voucher);
      }
    });
    finchCheckout.init();
  };
</script>
```

After a user has successfully completed the payment, `onSuccess` callback will be called and you’ll receive a payment voucher (JSON Web Token) as a parameter. Send the voucher to your service’s backend so that you can decode and verify it.
Please refer to the [official documentation](https://docs.finchtech.io/docs/getting_started/payment_verification) for a more detailed explanation of our payment voucher.

## Store Management Console

We also provide an open-source web-based [store management console](https://github.com/finch-tech/finch-management-console).

## Resources

- [Documentation](https://docs.finchtech.io/docs/home/overview.html)
- [Installation Guide](https://docs.finchtech.io/docs/installation/server)
- [Getting Started Guide](https://docs.finchtech.io/docs/getting_started/overview) (Store Setup and Integration)
- [Payment API Documentation](https://docs.finchtech.io/docs/payment_api/payments/create)
- [Store Management API Documentation](https://docs.finchtech.io/docs/management_api/auth/registration)
