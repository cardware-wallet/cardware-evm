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
use ethers::providers::{Http, Provider};
use ethers::providers::Middleware;
use std::convert::TryFrom;
use serde::{Deserialize, Serialize};
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;

#[wasm_bindgen]
pub struct Wallet{
    infura_url: String,
    xpub : String,
    address: String,
    chain_id: u64,
    nonce: u64,
    eth_balance : f64,
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
            eth_balance:0.0,
            balance: U256::zero(),
            gas_price: U256::zero(),
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
                        self.balance = balance;
                        self.eth_balance = wei_to_eth(self.balance);
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
                        self.gas_price = gas_price;
                    },
                    _ => {}
                }
            }
            "Sync successful.".to_string()
        } else {
            "Error: Infura error.".to_string()
        }
    }

    pub fn send(&self,to: String,value: &str,data: Option<String>,) -> String {
        // Convert the value from a decimal string to U256
        let gas_limit : u64= 21000;
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

        let mut hasher = Keccak::v256();
        let mut tx_hash = [0u8; 32];
        hasher.update(&rlp_encoded);
        hasher.finalize(&mut tx_hash);

        // Return the unsigned transaction as a hex string.
        let final_str = base64::encode(&rlp_encoded) + ":&" + &base64::encode(&tx_hash);
        //let final_str = &base64::encode(&tx_hash);
        //return chunk_and_label(&final_str,40);
        return final_str;
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
	    self.address = format!("0x{}", hex::encode(address));
    	return format!("0x{}", hex::encode(address));
    }
    pub fn balance(&self) -> String {
        self.eth_balance.to_string()
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
pub fn wei_to_eth(wei: U256) -> f64 {
    // Convert U256 to a string, then parse as f64.
    // Note: This approach works well for typical balances,
    // but may lose precision for extremely large values.
    let wei_str = wei.to_string();
    let wei_f64: f64 = wei_str.parse().expect("Failed to parse Wei as f64");
    // Divide by 10^18 to get Ether
    wei_f64 / 1e18
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


#[derive(Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u32,
    // The result is a hexadecimal string representing the balance in wei
    result: Option<String>,
    error: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: Vec<&'a str>,
    id: u32,
}

