use wasm_bindgen::prelude::*;
use reqwest::Client;
use serde_json::json;
use hex;
use rlp::RlpStream;
use std::str::FromStr;
use ethereum_types::{Address, U256};
use bitcoin::bip32::Xpub;
use bitcoin::bip32::DerivationPath;
use bitcoin::PublicKey;
use tiny_keccak::Keccak;
use tiny_keccak::Hasher;

#[wasm_bindgen]
pub struct Wallet{
    infura_url: String,
    xpub : String,
    address: String,
    chain_id: u64,
    nonce: u64,
    balance: U256,
    gas_price: U256,
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(constructor)]
    pub fn new(xpub: String, infura_url: String, chain_id: u64) -> Wallet {
        Wallet {
            infura_url,
            xpub,
            address:"".to_string(),
            chain_id,
            nonce: 0,
            balance: U256::zero(),
            gas_price: U256::zero(),
        }
    }

    pub async fn sync(&mut self) -> String {
        let client = Client::new();

        // Fetch balance
        let balance_req = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [self.address, "latest"],
            "id": 1
        });
        let balance_resp = client
            .post(&self.infura_url)
            .json(&balance_req)
            .send()
            .await;
        if let Ok(resp) = balance_resp {
            if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                if let Some(bal_hex) = resp_json.get("result").and_then(|r| r.as_str()) {
                    // Parse the hex string (e.g. "0x38d7ea4c68000")
                    self.balance = U256::from_str_radix(&bal_hex.trim_start_matches("0x"), 16)
                        .unwrap_or(U256::zero());
                }
            }
        } else {
            return "Error: Failed to fetch balance.".to_string();
        }

        // Fetch nonce (transaction count)
        let nonce_req = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [self.address, "latest"],
            "id": 1
        });
        let nonce_resp = client
            .post(&self.infura_url)
            .json(&nonce_req)
            .send()
            .await;
        if let Ok(resp) = nonce_resp {
            if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                if let Some(nonce_hex) = resp_json.get("result").and_then(|r| r.as_str()) {
                    self.nonce = u64::from_str_radix(&nonce_hex.trim_start_matches("0x"), 16)
                        .unwrap_or(0);
                }
            }
        } else {
            return "Error: Failed to fetch nonce.".to_string();
        }

        // Fetch current gas price
        let gas_req = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });
        let gas_resp = client
            .post(&self.infura_url)
            .json(&gas_req)
            .send()
            .await;
        if let Ok(resp) = gas_resp {
            if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                if let Some(gas_hex) = resp_json.get("result").and_then(|r| r.as_str()) {
                    self.gas_price = U256::from_str_radix(&gas_hex.trim_start_matches("0x"), 16)
                        .unwrap_or(U256::zero());
                }
            }
        } else {
            return "Error: Failed to fetch gas price.".to_string();
        }
        "Sync successful.".to_string()
    }

    pub fn send(&self,to: String,value: &str,gas_limit: u64,data: Option<String>,) -> String {
        // Convert the value from a decimal string to U256
        let value_u256 = U256::from_dec_str(value).unwrap_or(U256::zero());
        // Decode the data payload from hex if provided
        let data_bytes = match data {
            Some(d) => {
                let trimmed = d.trim_start_matches("0x");
                hex::decode(trimmed).unwrap_or_else(|_| Vec::new())
            }
            None => Vec::new(),
        };

        // RLP encode the transaction according to EIP-155.
        // The list of fields for signing is:
        // [ nonce, gas_price, gas_limit, to, value, data, chain_id, 0, 0 ]
        let mut stream = RlpStream::new_list(9);
        stream.append(&U256::from(self.nonce));
        stream.append(&self.gas_price);
        stream.append(&U256::from(gas_limit));
        let to_address = Address::from_str(&to).unwrap_or(Address::zero());
        stream.append(&to_address);
        stream.append(&value_u256);
        stream.append(&data_bytes);
        stream.append(&self.chain_id);
        stream.append(&0u8);
        stream.append(&0u8);
        let rlp_encoded = stream.out();

        // Return the unsigned transaction as a hex string.
        hex::encode(rlp_encoded)
    }

    pub async fn broadcast(&mut self, signed_tx: String) -> String {

        let tx_hex = if let Ok(decoded) = base64::decode(&signed_tx) {
            hex::encode(decoded)
        } else {
            signed_tx
        };

        let client = Client::new();
        let req_body = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [format!("0x{}", tx_hex)],
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
        "Error: Failed to broadcast transaction.".to_string()
    }
    pub fn address(&mut self) -> String{
    	let xpub_tmp_str = &convert_to_xpub(self.xpub.clone()); //Xpub 1 
        let xpub = match Xpub::from_str(&xpub_tmp_str){
            Ok(xpub) => xpub,
            Err(_) => return "Error: Xpub derivation error.".to_string(),
        };
        let derivation_path = DerivationPath::from_str("m/0/0").unwrap();
        let derived_xpub = match xpub.derive_pub(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path){
            Ok(derived_xpub) => derived_xpub,
            Err(_) => return "Error: Xpub derivation error.".to_string(),
        };
        let public_key = PublicKey::new(
            derived_xpub.public_key
        );
        let uncompressed = public_key.to_bytes();
	    // Skip the first byte (0x04)
	    let public_bytes = &uncompressed[1..];
	    // Hash with Keccak-256
	    let mut hasher = Keccak::v256();
	    let mut output = [0u8; 32];
	    hasher.update(public_bytes);
	    hasher.finalize(&mut output);
	    // Take the lower 20 bytes for the address
	    let address = &output[12..];
	    
    	return format!("0x{}", hex::encode(address));
    }
    pub fn balance(&self) -> String {
        self.balance.to_string()
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
    Some(bytes)
}
