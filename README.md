# Cardware EVM NPM Library

This is the documentation for the EVM NPM package that communicates with the Cardware device.

The library requires an rpc endpoint to query an EVM based blockchain.

It allows users to create a watch-only wallet on the web.

All data that is transferred between the web wallet and the Cardware device is done through scanning QR codes.

Users must first pair the web wallet with their Cardware device.

Once paired they are then able to view the EVM address of their Cardware device, see their confirmed and unconfirmed EVM balances and send EVM tokens from their Cardware device.

When sending, the watch only wallet will create an unsigned transaction which will be split up into QR codes. The user will then be prompted to scan these QR codes with their Cardware device. The user will then confirm the transactions details which will then create a signed transaction which their Cardware device will split up into QR codes. The web wallet then scans these QR codes, decodes them and broadcasts the transaction.

---
