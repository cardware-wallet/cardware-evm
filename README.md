# Cardware EVM NPM Library

This is the documentation for the EVM NPM package that communicates with the Cardware device.

The library requires an rpc endpoint to query an EVM based blockchain.

It allows users to create a watch-only wallet on the web.

All data that is transferred between the web wallet and the Cardware device is done through scanning QR codes.

Users must first pair the web wallet with their Cardware device.

Once paired they are then able to view the EVM address of their Cardware device, see their confirmed and unconfirmed EVM balances and send EVM tokens from their Cardware device.

When sending, the watch only wallet will create an unsigned transaction which will be split up into QR codes. The user will then be prompted to scan these QR codes with their Cardware device. The user will then confirm the transactions details which will then create a signed transaction which their Cardware device will split up into QR codes. The web wallet then scans these QR codes, decodes them and broadcasts the transaction.

---

# Documentation

## Initialization

### Code

```javascript
import Wallet from 'cardware-evm'; 
```

---

## New Wallet

This function initializes a wallet object in your web wallet. The zpub is received from the Cardware device after successfully pairing the web wallet and Cardware device. The pairing process involves scanning the **pair** QR codes from the Cardware device, extracting the zpub, then using it in creating the wallet object.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| zpub | string | The zpub of the the hardware wallet. | ```"zpub5ZNhc5KKM6hACK6QDuo6UG1749XUeXf9Gbu8rcZQnNDeMJwUPrwzEVKsF7X7EzZe5yqwymfMA1tGJ9qAmjdmGHSkRW7SruCEDz9mgEkwWvN"``` |
| account_derivation_path | string | The derivation path of the wallet. | ```"m/0/0"``` |
| esplora_url | string | The address of the esplora you are using. | ```"https://mainnet.infura.io/v3/API_KEY"``` |
| chain_id | BigInt | The chain id of the chain you are using. | ```1``` |

### Code

```javascript
var wallet = await new Wallet(zpub, account_derivation_path, esplora_url, chain_id);
```

### Output

No outputs.

---

## Sync

This function syncs your web wallet to make sure it has all the correct information to be able to get balances, construct unsigned transactions and broadcast transactions.

### Parameters

No parameters.

### Code

```javascript
await wallet.sync();
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The wallet has synced successfully. | ```"Sync successful."``` |
| error | There is an issue connecting to the infura node. | ```"Error: Infura error."``` |
| error | There is an issue parsing the response JSON. | ```"Error: JSON parse error."``` |
| error | There is an issue with the format of the JSON. | ```"Error: Unexpected JSON format."``` |
| error | There is an issue parsing the balance. | ```"Error: Balance parse error."``` |
| error | There is an issue parcing the nonce. | ```"Error: Error: Nonce parse error."``` |
| error | There is an issue parsing the gas price. | ```"Error: Gas price parse error"``` |

---

## Estimate Fees

This function estimates fees for a send transaction which takes a variable called **fee rate** where the higher the fee rate, the higher the estimated fee and the faster the transaction will confirm. Fee rate has 3 options, either 1, 2 or 3. The other variable is called **gas limit** and sets what the upper bound of the fee must be.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| fee_rate | int32 | The rate which decides the tx fee. 0 is slow, 1 is medium and 2 is fast. | ```2``` |
| gas_limit | int32 | The upper bound of the fee. | ```21000``` |

### Code

```javascript
let result = wallet.estimate_fee(fee_rate, gas_limit);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The fee estimation for a transaction. | ```"500256483"``` |

---

## Send

This function creates an unsigned transaction. It puts splits it up into chunks to be put converted into QR codes. At the beginning of the chunk extra information is added. The extra information has the format of *(* + *index of QR code* + */* + *total QR codes* + *)* + *the unsigned transaction*.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| to | string | The address to send to. | ```"0x02A8665a18BBa2D1B4766e2D71977A781b97592e"``` |
| value | string | The send amount with the correct decimals (example uses 18 decimals). | ```200000000000000``` |
| fee_rate | int32 | The rate which decides the tx fee. 0 is slow, 1 is medium and 2 is fast. | ```2``` |

### Code

```javascript
var qrcode_chunks = wallet.send(to, value, fee_rate);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The unsigned transaction and the signature seperated by a **:**. | ```""``` |
| error | The is an issue with the derivation path. | ```"Error: Derivation path error."``` |

---

## Send (ERC20)

This function creates an unsigned transaction. It puts splits it up into chunks to be put converted into QR codes. At the beginning of the chunk extra information is added. The extra information has the format of *(* + *index of QR code* + */* + *total QR codes* + *)* + *the unsigned transaction*.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| contract_address | string | The contract of the ERC20 you wish to send. | ```"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"``` |
| recipient | string | The address to send to. | ```"0x02A8665a18BBa2D1B4766e2D71977A781b97592e"``` |
| value | string | The send amount with the correct decimals (example uses 6 decimals). | ```2000000``` |
| fee_rate | int32 | The rate which decides the tx fee. 0 is slow, 1 is medium and 2 is fast. | ```2``` |

### Code

```javascript
var qrcode_chunks = wallet.erc20_transfer(contract_address, recipient, value, fee_rate);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The unsigned transaction and the transaction signature seperated by a **:**. | ```""``` |
| error | The is an issue with the derivation path. | ```"Error: Derivation path error."``` |

---

## Broadcast

This function needs an unsigned transaction and a transaction signature. It gets this by scanning the QR codes on the Cardware device. When scanning the QR codes of the signed transaction from the Cardware device it follows the format of *(* + *index of QR code* + */* + *total QR codes* + *)* + *the unsigned transaction + the transaction signature*.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| unsigned_tx | string | The unsigned transaction in base64 that needs to be broadcasted. | ```""``` |
| tx_signature | string | The transaction signature in base64 that needs to be broadcasted. | ```""``` |

### Code

```javascript
await wallet.broadcast(unsigned_tx, tx_signature);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The transaction ID of the broadcasted transaction. | ```"0x4038c7f2a5b7ce726e67f64f0604cf147cf1fcc15fd29c77988e486e8eab0da9"``` |
| error | There is an issue decoding the unsigned transaction. | ```"Error: Failed to decode the unsigned transaction."```
| error | There is an issue decoding the transaction signature. | ```"Error: Failed to decode the transaction signature."```
| error | There is an issue decoding the nonce. | ```"Error: Failed to decode the nonce."```
| error | There is an issue decoding the gas price. | ```"Error: Failed to decode the gas price."```
| error | There is an issue decoding the gas limit. | ```"Error: Failed to decode the gas limit."```
| error | There is an issue decoding the output. | ```"Error: Failed to decode the output."```
| error | There is an issue decoding the value. | ```"Error: Failed to decode the value."```
| error | There is an issue decoding the field. | ```"Error: Failed to decode the data field."```
| error | There is an issue decoding the chain ID. | ```"Error: Failed to decode the chain ID."```
| error | There is an issue broadcasting the transaction. | ```"Error: Failed to broadcast transaction."```

---

## Address

This function returns the address of your Cardware device at the given derivation path that the wallet was initialised with.

### Parameters

No parameters.

### Code

```javascript
const result = wallet.address();
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The address of the wallet. | ```"0x128f5DeF395f5587744dfeC2b154bD618415d769"``` |
| error | There is an issue deriving the zPub. | ```"Error: zPub derivation error."``` |

---

## Balance

This function returns confirmed balance of your Cardware device.

### Parameters

No parameters.

### Code

```javascript
const result = wallet.balance();
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The confirmed balance of the native token for your wallet. | ```"0.003739700213554025"``` |
---

## Balance (ERC20)

This function returns confirmed balance of a list of ERC20 tokens for your Cardware device.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| contract_address | array[string] | A list of contract addresses of the ERC20 token. | ```["0xdAC17F958D2ee523a2206206994597C13D831ec7"]``` |

### Code

```javascript
const result = wallet.erc20_balance(contract_address);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The confirmed balance of a specific ERC20 token for your wallet.| ```"0.003739700213554025"``` |
---

## Validate Contract (ERC20)

This function validates an ERC20 contract when given a contract address.

### Parameters

| Parameter | Type | Description | Example |
|---|---|---|---|
| contract_address | array[string] | The contract address of the ERC20 token. | ```"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"``` |

### Code

```javascript
const result = wallet.validate_contract(contract_address);
```

### Output

The output is a string.

| Result | Description | Output |
|---|---|---|
| success | The confirmed balance of a specific ERC20 token for your wallet.| ```"{"address":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","decimals":6,"name":"USD Coin","symbol":"USDC"}"``` |
| error | There is an isuue with the esplora. | ```"Error: Infura error during batch request."``` |
| error | There is an issue with parsing the JSON. | ```"Error: JSON parse error during batch request."``` |
| error | There is an issue with the JSON format. | ```"Error: Unexpected JSON format in batch response."``` |
| error | The decimals value is out of range. | ```"Error: Decimals value out of range."``` |
| error | There is an issue decoding the decimals. | ```"Error: Failed to decode decimals."``` |
| error | There is an issue decoding the symbol. | ```"Error: Failed to decode symbol."``` |
| error | There is an issue decoding the name. | ```"Error: Failed to decode name."``` |

---
