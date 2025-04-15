use wasm_bindgen::prelude::*;
use reqwest::Client;
use serde_json::json;
use hex;
use rlp::RlpStream;
use rlp::Rlp;
use std::str::FromStr;
use ethereum_types::{Address, U256};
use bitcoin::bip32::Xpub;
use bitcoin::bip32::DerivationPath;
use bitcoin::PublicKey;
use tiny_keccak::Keccak;
use tiny_keccak::Hasher;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;

#[wasm_bindgen]
pub struct Wallet{
    infura_url: String,
    xpub : String,
    account_derivation_path : String,
    address: String,
    chain_id: u64,
    nonce: u64,
    eth_balance : f64,
    balance: String,
    gas_price: String,
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(constructor)]
    pub fn new(xpub: String, account_derivation_path : String, infura_url: String, chain_id: u64) -> Wallet { //Acount derivation paths must be in the format m/x/y eg: "m/0/0" or "m/1/2"
        Wallet {
            infura_url,
            xpub,
            account_derivation_path,
            address: "".to_string(),
            chain_id,
            nonce: 0,
            eth_balance: 0.0,
            balance: "0".to_string(),
            gas_price: "0".to_string(),
        }
    }
    pub async fn sync(&mut self) -> String {
        let url = self.infura_url.clone();
        let client = reqwest::Client::new();
        let addr = self.address(); 

        // Batch JSON-RPC request
        let request_body = json!([
            {
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": [addr.clone(), "latest"],
                "id": 1,
            },
            {
                "jsonrpc": "2.0",
                "method": "eth_getTransactionCount",
                "params": [addr.clone(), "latest"],
                "id": 2,
            },
            {
                "jsonrpc": "2.0",
                "method": "eth_gasPrice",
                "params": [],
                "id": 3,
            }
        ]);

        let response = match client.post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await {
                Ok(resp) => resp,
                Err(_) => return "Error: Infura error.".to_string(),
            };

        if response.status().is_success() {
            let body = response.text().await.unwrap();
            let parsed: Value = match serde_json::from_str(&body) {
                Ok(val) => val,
                Err(_) => return "Error: JSON parse error.".to_string(),
            };

            let responses = match parsed.as_array() {
                Some(arr) => arr,
                None => return "Error: Unexpected JSON format.".to_string(),
            };

            for resp in responses {
                let id = resp["id"].as_i64().unwrap_or_default();
                let result = match resp["result"].as_str() {
                    Some(r) => r,
                    None => continue,
                };
                match id {
                    1 => { // eth_getBalance
                        let balance = match U256::from_str_radix(result.trim_start_matches("0x"), 16) {
                            Ok(val) => val,
                            Err(_) => return "Error: Balance parse error.".to_string(),
                        };
                        self.balance = gas_price_to_string(balance);
                        self.eth_balance = wei_to_eth(balance);
                    },
                    2 => { // eth_getTransactionCount (nonce)
                        let nonce = match U256::from_str_radix(result.trim_start_matches("0x"), 16) {
                            Ok(val) => val,
                            Err(_) => return "Error: Nonce parse error.".to_string(),
                        };
                        self.nonce = nonce.low_u64();
                    },
                    3 => { // eth_gasPrice
                        let gas_price = match U256::from_str_radix(result.trim_start_matches("0x"), 16) {
                            Ok(val) => val,
                            Err(_) => return "Error: Gas price parse error.".to_string(),
                        };
                        self.gas_price = gas_price_to_string(gas_price);
                    },
                    _ => {}
                }
            }
            return "Sync successful.".to_string();
        } else {
            return "Error: Infura error.".to_string();
        }
    }
    //fee rate determines tx fee, 0 = slow, 1 = medium, 2 = fast
    pub fn send(&self, to: String, value: &str, fee_rate : i32) -> String {
        // Convert the value from a decimal string to U256
        let gas_limit : u64 = 21000;
        let value_u256 = U256::from_dec_str(value).unwrap_or(U256::zero());
        let new_gas_price;
        let self_gas = gas_price_from_string(&self.gas_price);
        match fee_rate{
            0 => new_gas_price = &self_gas * U256::from(10) / U256::from(10),
            1 => new_gas_price = self_gas * U256::from(15) / U256::from(10),
            2 => new_gas_price = &self_gas * U256::from(20) / U256::from(10),
            _ => new_gas_price = self_gas,
        }

        // RLP encode the transaction according to EIP-155.
        // The list of fields for signing is:
        // [ nonce, gas_price, gas_limit, to, value, data, chain_id, 0, 0 ]

        let mut stream = RlpStream::new_list(9);
        stream.append(&U256::from(self.nonce));
        stream.append(&new_gas_price);
        stream.append(&U256::from(gas_limit));
        let to_address = Address::from_str(&to).unwrap_or(Address::zero());
        stream.append(&to_address);
        stream.append(&value_u256);
        stream.append(&Vec::new()); //Data bytes
        stream.append(&self.chain_id);
        stream.append(&0u8);
        stream.append(&0u8);
        let rlp_encoded = stream.out();
        let mut hasher = Keccak::v256();
        let mut tx_hash = [0u8; 32];
        hasher.update(&rlp_encoded);
        hasher.finalize(&mut tx_hash);
        let mut total_bytes : Vec<u8> = Vec::new();
        total_bytes.extend_from_slice(&tx_hash);
        match extract_u16s(&self.account_derivation_path) {
            Ok((first, second)) => append_integers_as_bytes(&mut total_bytes,first,second),
            Err(_) => return "Error: Derivation path error.".to_string(),
        }
        let unsigned_tx = hex::encode(rlp_encoded);

        // Return the unsigned transaction as a hex string.
        let final_str = unsigned_tx + ":&" + &base64::encode(&total_bytes);
        return final_str;
    }
    pub async fn validate_contract(&mut self, contract_address: String) -> String {
        let url = self.infura_url.clone();
        let client = reqwest::Client::new();

        // Batch JSON-RPC requests for decimals, symbol, and name.
        let batch_request = json!([
            {
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": contract_address.clone(),
                    "data": "0x313ce567" // decimals()
                }, "latest"],
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": contract_address.clone(),
                    "data": "0x95d89b41" // symbol()
                }, "latest"],
                "id": 2
            },
            {
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": contract_address.clone(),
                    "data": "0x06fdde03" // name()
                }, "latest"],
                "id": 3
            }
        ]);

        let response = match client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&batch_request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) => return "Error: Infura error during batch request.".to_string(),
        };

        if !response.status().is_success() {
            return "Error: Infura error during batch request.".to_string();
        }

        let body = response.text().await.unwrap();
        let responses: Value = match serde_json::from_str(&body) {
            Ok(val) => val,
            Err(_) => return "Error: JSON parse error during batch request.".to_string(),
        };

        let responses_array = match responses.as_array() {
            Some(arr) => arr,
            None => return "Error: Unexpected JSON format in batch response.".to_string(),
        };

        // Initialize empty strings for each response.
        let mut decimals_hex = "";
        let mut symbol_hex = "";
        let mut name_hex = "";

        // Iterate through responses, matching each by its id.
        for resp in responses_array {
            if let Some(id) = resp.get("id").and_then(|v| v.as_i64()) {
                match id {
                    1 => {
                        decimals_hex = resp.get("result").and_then(|r| r.as_str()).unwrap_or("");
                    },
                    2 => {
                        symbol_hex = resp.get("result").and_then(|r| r.as_str()).unwrap_or("");
                    },
                    3 => {
                        name_hex = resp.get("result").and_then(|r| r.as_str()).unwrap_or("");
                    },
                    _ => {},
                }
            }
        }

        // Decode decimals using hex_to_vec directly.
        let decimals = match hex_to_vec(decimals_hex.trim_start_matches("0x")) {
            Some(mut bytes) => {
                // Remove any leading zeros.
                while !bytes.is_empty() && bytes[0] == 0 {
                    bytes.remove(0);
                }
                let value = if bytes.is_empty() {
                    0u64
                } else {
                    bytes.into_iter().fold(0u64, |acc, b| acc * 256 + b as u64)
                };
                if value <= u8::MAX as u64 {
                    value as u8
                } else {
                    return "Error: Decimals value out of range.".to_string();
                }
            },
            None => return "Error: Failed to decode decimals.".to_string(),
        };

        let symbol = match decode_abi_string(symbol_hex) {
            Some(s) => s,
            None => return "Error: Failed to decode symbol.".to_string(),
        };

        let name = match decode_abi_string(name_hex) {
            Some(n) => n,
            None => return "Error: Failed to decode name.".to_string(),
        };

        // Assemble the contract data into a JSON object and return it as a string.
        let contract_data = json!({
             "address": contract_address,
             "decimals": decimals,
             "symbol": symbol,
             "name": name,
        });
        return contract_data.to_string();
    }
    pub fn erc20_transfer(&self, contract_address: String, recipient: String, token_amount: &str, fee_rate: i32) -> String {
        // Use a higher gas limit for token transfers.
        let gas_limit: u64 = 160000;
        let token_amount_u256 = U256::from_dec_str(token_amount).unwrap_or(U256::zero());
        // Encode the ERC20 transfer data.
        let data = encode_transfer(&recipient, token_amount_u256);
        let new_gas_price;
        let self_gas = gas_price_from_string(&self.gas_price);
         match fee_rate{
            0 => new_gas_price = &self_gas * U256::from(10) / U256::from(10),
            1 => new_gas_price = self_gas * U256::from(15) / U256::from(10),
            2 => new_gas_price = &self_gas * U256::from(20) / U256::from(10) ,
            _ => new_gas_price = self_gas,
        }

        let mut stream = RlpStream::new_list(9);
        stream.append(&U256::from(self.nonce));
        stream.append(&new_gas_price);
        stream.append(&U256::from(gas_limit));
        let contract_addr = Address::from_str(&contract_address).unwrap_or(Address::zero());
        stream.append(&contract_addr);
        // For token transfers, ETH value is zero.
        stream.append(&U256::zero());
        stream.append(&data);
        stream.append(&self.chain_id);
        stream.append(&0u8);
        stream.append(&0u8);
        let rlp_encoded = stream.out();
        let mut hasher = Keccak::v256();
        let mut tx_hash = [0u8; 32];
        hasher.update(&rlp_encoded);
        hasher.finalize(&mut tx_hash);
        let mut total_bytes : Vec<u8> = Vec::new();
        total_bytes.extend_from_slice(&tx_hash);
        match extract_u16s(&self.account_derivation_path) {
            Ok((first, second)) => append_integers_as_bytes(&mut total_bytes,first,second),
            Err(_) => return "Error: Derivation path error.".to_string(),
        }
        let unsigned_tx = hex::encode(rlp_encoded);
        let final_str = unsigned_tx + ":&" + &base64::encode(&total_bytes);
        return final_str;
    }
    //This function now always accepts and returns a list of balances for a list of contracts
    pub async fn erc20_balance(&self, contract_addresses: Vec<String>) -> Vec<String> {
        // Clean and pad the wallet address
        let wallet_addr_clean = self.address.trim_start_matches("0x");
        let padded_wallet_addr = format!("{:0>64}", wallet_addr_clean);

        // Build the call data using the ERC20 balanceOf selector (0x70a08231)
        let call_data = format!("0x70a08231{}", padded_wallet_addr);

        // Build batched JSON-RPC requests, one per contract address
        let mut batch_requests = Vec::new();
        for (i, contract_address) in contract_addresses.iter().enumerate() {
            let req = json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [
                    {
                        "to": contract_address,
                        "data": call_data
                    },
                    "latest"
                ],
                "id": i + 1  // Assign a unique id for each request
            });
            batch_requests.push(req);
        }

        // Send the batched request to Infura
        let client = reqwest::Client::new();
        let response = match client
            .post(&self.infura_url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&batch_requests)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(_) => return vec!["Error: Infura error.".to_string()],
        };

        if response.status().is_success() {
            let body = match response.text().await {
                Ok(text) => text,
                Err(_) => return vec!["Error: Infura error.".to_string()],
            };

            // Parse the response as an array of JSON objects
            let parsed: Vec<serde_json::Value> = match serde_json::from_str(&body) {
                Ok(val) => val,
                Err(_) => return vec!["Error: JSON parse error.".to_string()],
            };

            let mut balances = Vec::new();
            // Process each response object in the batch
            for resp_item in parsed {
                let result_str = match resp_item.get("result").and_then(|r| r.as_str()) {
                    Some(r) => r,
                    None => return vec!["Error: Unexpected JSON format.".to_string()],
                };

                let balance_u256 = match U256::from_str_radix(result_str.trim_start_matches("0x"), 16) {
                    Ok(val) => val,
                    Err(_) => return vec!["Error: Balance parse error.".to_string()],
                };

                balances.push(balance_u256.to_string());
            }
            // Prepend "Success" as the first element
            let mut result_vec = vec!["Success".to_string()];
            result_vec.extend(balances);
            result_vec
        } else {
            vec!["Error: Infura error.".to_string()]
        }
    }
    pub async fn broadcast(&mut self, unsigned_tx: String, tx_signature : String) -> String {
        let unsigned_tx_hex = unsigned_tx.trim_start_matches("0x");
        let unsigned_tx_bytes = match hex::decode(unsigned_tx_hex){
            Ok(bytes) => bytes,
            Err(_) => return "Error: Failed to decode the unsigned transaction.".to_string(),
        };

        // Decode the unsigned transaction RLP.
        // This unsigned tx is expected to have 9 fields:
        // [nonce, gasPrice, gasLimit, to, value, data, v, r, s]
        // In the unsigned tx, the v, r, s fields are placeholders (usually 0x80).
        let rlp_unsigned = Rlp::new(&unsigned_tx_bytes);
        let base_bytes = match base64::decode(&tx_signature){
            Ok(bytes) => bytes,
            Err(_) => return "Error: Failed to decode the transaction signature.".to_string()
        };

        let nonce = match rlp_unsigned.at(0) {
            Ok(field) => match field.as_val::<U256>() {
                Ok(val) => val,
                Err(_) => return "Error: Failed to decode the nonce.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the nonce.".to_string(),
        };

        let gas_price = match rlp_unsigned.at(1) {
            Ok(field) => match field.as_val::<U256>() {
                Ok(val) => val,
                Err(_) =>return "Error: Failed to decode the gas price.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the gas price.".to_string(),
        };

        let gas_limit = match rlp_unsigned.at(2) {
            Ok(field) => match field.as_val::<U256>() {
                Ok(val) => val,
                Err(_) => return "Error: Failed to decode the gas limit.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the gas limit.".to_string(),
        };

        let to = match rlp_unsigned.at(3) {
            Ok(field) => match field.data() {
                Ok(data) => data.to_vec(),
                Err(_) => return "Error: Failed to decode the output.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the output.".to_string(),
        };

        let value = match rlp_unsigned.at(4) {
            Ok(field) => match field.as_val::<U256>() {
                Ok(val) => val,
                Err(_) => return "Error: Failed to decode the value.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the value.".to_string(),
        };

        let data_field = match rlp_unsigned.at(5) {
            Ok(field) => match field.data() {
                Ok(data) => data.to_vec(),
                Err(_) => return "Error: Failed to decode the data field.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the data field.".to_string(),
        };

        let chain_id = match rlp_unsigned.at(6) {
            Ok(field) => match field.as_val::<U256>() {
                Ok(val) => val,
                Err(_) => return "Error: Failed to decode the chain ID.".to_string(),
            },
            Err(_) => return "Error: Failed to decode the chain ID.".to_string(),
        };

        let r_sig = &base_bytes[0..32];
        let s_sig = &base_bytes[32..64];
        let v_sig = base_bytes[64];
        
        let recovery_id = if v_sig > 1 { v_sig - 27 } else { v_sig };
        let v_eip155 = chain_id.low_u64() * 2 + 35 + recovery_id as u64;
        let mut stream = RlpStream::new_list(9);
        stream.append(&nonce);
        stream.append(&gas_price);
        stream.append(&gas_limit);
        stream.append(&to);
        stream.append(&value);
        stream.append(&data_field);
        stream.append(&v_eip155);
        stream.append(&r_sig);
        stream.append(&s_sig);

        let signed_tx_bytes = stream.out().to_vec();
        let signed_tx_hex = format!("0x{}", hex::encode(&signed_tx_bytes));
        
        let client = Client::new();
        let req_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [signed_tx_hex],
            "id": 1
        });
        let resp = client.post(&self.infura_url).json(&req_body).send().await;
        if let Ok(response) = resp {
            if let Ok(resp_json) = response.json::<serde_json::Value>().await {
                if let Some(result) = resp_json.get("result").and_then(|r| r.as_str()) {
                    return result.to_string();
                } else if let Some(error) = resp_json.get("error") {
                    return format!("Error: {:?}", error);
                }
            }
        }
        return "Error: Failed to broadcast transaction.".to_string();
    }
    pub fn address(&mut self) -> String{
    	let xpub_tmp_str = &convert_to_xpub(self.xpub.clone()); //Xpub 1
        let xpub = match Xpub::from_str(&xpub_tmp_str){
            Ok(xpub) => xpub,
            Err(_) => return "Error: zPub derivation error.".to_string(),
        };
        let derivation_path = DerivationPath::from_str(&self.account_derivation_path).unwrap();
        let derived_xpub = match xpub.derive_pub(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path){
            Ok(derived_xpub) => derived_xpub,
            Err(_) => return "Error: zPub derivation error.".to_string(),
        };
        let public_key = PublicKey::new_uncompressed(
            derived_xpub.public_key
        );
        let uncompressed = public_key.to_bytes();
	    let public_bytes = &uncompressed[1..];
	    // Hash with Keccak-256
	    let mut hasher = Keccak::v256();
	    let mut output = [0u8; 32];
	    hasher.update(public_bytes);
	    hasher.finalize(&mut output);
	    let address = &output[12..];
	    self.address = format!("0x{}", hex::encode(address));
    	return format!("0x{}", hex::encode(address));
    }
    pub fn balance(&self) -> String {
        self.eth_balance.to_string()
    }
    //fee rate, 0 = slow, 1 = medium, 2 = fast
    pub fn estimate_fee(&self, fee_rate : i32, gas_limit : i32) -> String{
        let mut new_gas_price;
        let self_gas = gas_price_from_string(&self.gas_price);
        match fee_rate{
            0 => new_gas_price = &self_gas * U256::from(10) / U256::from(10),
            1 => new_gas_price = &self_gas * U256::from(15) / U256::from(10),
            2 => new_gas_price = &self_gas * U256::from(20) / U256::from(10),
            _ => new_gas_price = self_gas,
        }
        new_gas_price = new_gas_price * U256::from(gas_limit);
        return format!("{}", wei_to_eth(new_gas_price));
    }
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

pub fn convert_to_xpub(xpub_str : String) -> String{
    let zpub_bytes = match bs58::decode(&xpub_str).with_check(None).into_vec(){
        Ok(zpub_bytes) => zpub_bytes,
        Err(_) => return "Error: Invalid extended public key.".to_string(),
    };
    let new_bytes = &zpub_bytes[4..];
    let new_prefix = hex_to_vec("0488b21e").unwrap();
    let mut vec = Vec::from(new_bytes);
    for i in (0..new_prefix.len()).rev() {
        vec.insert(0, new_prefix[i]);
    }
    return bs58::encode(vec).with_check().into_string();
}
fn gas_price_to_string(gas_price: U256) -> String {
    return gas_price.to_string();
}
// Convert a decimal string to a U256 gas price.
fn gas_price_from_string(s: &str) -> U256 {
    let res = match U256::from_dec_str(s){
        Ok(res) => res,
        Err(_) => U256::zero(),
    };
    return res;
}
pub fn hex_to_vec(hex_string: &str) -> Option<Vec<u8>> {
    if hex_string.len() % 2 != 0 { return None; }
    let mut bytes = Vec::new();
    for chunk in hex_string.as_bytes().chunks(2) {
        if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16) {
            bytes.push(byte);
        }else{
            return None; 
        }
    }
    return Some(bytes);
}
fn decode_abi_string(hex_str: &str) -> Option<String> {
    let hex = hex_str.trim_start_matches("0x");
    if hex.len() >= 128 {
        // The first 64 characters are the offset; the next 64 characters contain the string length.
        let len_hex = &hex[64..128];
        let len = usize::from_str_radix(len_hex, 16).ok()?;
        let start = 128;
        let end = start + len * 2;
        if hex.len() < end {
            return None;
        }
        let data_hex = &hex[start..end];
        let bytes = hex_to_vec(data_hex)?;
        return String::from_utf8(bytes).ok();
    } else {
        let bytes = hex_to_vec(hex)?;
        let s = bytes.into_iter().take_while(|&b| b != 0).collect::<Vec<u8>>();
        return String::from_utf8(s).ok();
    }
}
pub fn wei_to_eth(wei: U256) -> f64 {
    // Convert U256 to a string, then parse as f64.
    // Note: This approach works well for typical balances,
    // but may lose precision for extremely large values.
    let wei_str = wei.to_string();
    let wei_f64: f64 = match wei_str.parse(){
        Ok(f6) => f6,
        Err(_) => return 0.0,
    };
    // Divide by 10^18 to get Ether
    return wei_f64 / 1e18;
}
pub fn chunk_and_label(final_str: &str, chunk_size: usize) -> Vec<String> {
    let total_chunks = (final_str.len() + chunk_size - 1) / chunk_size; // Calculate the number of chunks
    final_str
        .chars() // Iterate over characters to respect character boundaries
        .collect::<Vec<_>>() // Collect characters into a vector for chunking
        .chunks(chunk_size) // Chunk the vector
        .enumerate() // Provide index for each chunk
        .map(|(index, chunk)| {
            let chunk_str = chunk.iter().collect::<String>(); // Convert chunk to string
            format!("({}/{}){}", index, total_chunks, chunk_str) // Format with index and total
        })
        .collect() // Collect into a vector of strings
}
// Helper function to encode ERC20 transfer data.
pub fn encode_transfer(recipient: &str, amount: U256) -> Vec<u8> {
    let mut data = Vec::new();
    // Function selector for transfer(address,uint256): "a9059cbb"
    let selector = hex::decode("a9059cbb").expect("Invalid function selector");
    data.extend(selector);

    // Encode recipient address.
    let recipient_clean = recipient.trim_start_matches("0x");
    let recipient_bytes = hex::decode(recipient_clean).expect("Invalid recipient address");
    let mut padded_recipient = vec![0u8; 12]; // 32 - 20 = 12 bytes of zero padding.
    padded_recipient.extend(recipient_bytes);
    data.extend(padded_recipient);

    // Encode amount as a 32-byte big-endian integer.
    let mut amount_bytes = [0u8; 32];
    amount.to_big_endian(&mut amount_bytes);
    data.extend_from_slice(&amount_bytes);

    return data;
}
pub fn extract_u16s(input: &str) -> Result<(u16, u16), &'static str> {
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() != 3 {
            return Err("Error: Invalid format.");
        }
        let first_u16 = parts[1].parse::<u16>().map_err(|_| "Error: Failed to parse first number.")?;
        let second_u16 = parts[2].parse::<u16>().map_err(|_| "Error: Failed to parse second number.")?;
        return Ok((first_u16, second_u16))
}
pub fn append_integers_as_bytes(vec: &mut Vec<u8>, addressdepth: u16, changedepth: u16) {
    let addressdepth_bytes = addressdepth.to_le_bytes();
    let changedepth_bytes = changedepth.to_le_bytes();
    vec.extend_from_slice(&addressdepth_bytes);
    vec.extend_from_slice(&changedepth_bytes);
}

