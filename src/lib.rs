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
use ethers_core::abi::{AbiParser, Function, Token};
use sha3::{Digest, Keccak256};

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
    pub fn send_eip1559(&self, to: String, value: &str, fee_rate: i32) -> String {
        // 1) parse the value
        let value_u256 = match U256::from_dec_str(value) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to parse value.".to_string(),
        };

        // 2) gas settings
        let gas_limit: u64 = 21_000;
        let base_gas = match gas_price_from_string(&self.gas_price) {
            g => g,
        };
        let new_max_fee = match fee_rate {
            0 => base_gas.clone(),
            1 => base_gas.clone() * U256::from(15) / U256::from(10),
            2 => base_gas.clone() * U256::from(20) / U256::from(10),
            _ => base_gas.clone(),
        };
        let max_priority_fee = new_max_fee.clone();

        // 3) to address
        let to_addr = match Address::from_str(&to) {
            Ok(a) => a,
            Err(_) => Address::zero(),
        };

        // 4) RLP-encode [chain_id, nonce, maxPri, maxFee, gasLimit, to, value, data, accessList]
        let mut stream = RlpStream::new_list(9);
        stream.append(&self.chain_id);
        stream.append(&U256::from(self.nonce));
        stream.append(&max_priority_fee);
        stream.append(&new_max_fee);
        stream.append(&U256::from(gas_limit));
        stream.append(&to_addr);
        stream.append(&value_u256);
        stream.append(&Vec::<u8>::new());   // empty data
        stream.begin_list(0);               // empty accessList
        let rlp_payload = stream.out().to_vec();

        // 5) compute keccak256(0x02 || payload)
        let mut hasher = Keccak::v256();
        let mut sign_hash = [0u8; 32];
        hasher.update(&[0x02]);
        hasher.update(&rlp_payload);
        hasher.finalize(&mut sign_hash);

        // 6) append derivation for the HW wallet
        let mut to_sign = sign_hash.to_vec();
        match extract_u16s(&self.account_derivation_path) {
            Ok((h1, h2)) => append_integers_as_bytes(&mut to_sign, h1, h2),
            Err(_) => return "Error: Derivation path error.".to_string(),
        }

        // 7) return hex(payload) :& base64(sign_hash||derivation)
        let unsigned_hex = hex::encode(rlp_payload);
        let blob = base64::encode(&to_sign);
        format!("{}:&{}", unsigned_hex, blob)
    }
   pub fn prepare_eip1559(
    &self,
    to: String,
    value: String,
    max_priority_fee_per_gas: String,
    max_fee_per_gas: String,
    gas_limit: String,
    data: String
) -> String {
    // 1) Parse all the hex‐encoded numeric inputs:
    let value_u256 = match U256::from_str_radix(value.trim_start_matches("0x"), 16) {
        Ok(v) => v,
        Err(_) => return "Error: Failed to parse value.".to_string(),
    };
    let pri = match U256::from_str_radix(max_priority_fee_per_gas.trim_start_matches("0x"), 16) {
        Ok(v) => v,
        Err(_) => return "Error: Failed to parse max priority fee.".to_string(),
    };
    let fee = match U256::from_str_radix(max_fee_per_gas.trim_start_matches("0x"), 16) {
        Ok(v) => v,
        Err(_) => return "Error: Failed to parse max fee.".to_string(),
    };
    let gas_limit_u256 = match U256::from_str_radix(gas_limit.trim_start_matches("0x"), 16) {
        Ok(v) => v,
        Err(_) => return "Error: Failed to parse gas limit.".to_string(),
    };

    // 2) Parse the “to” address
    let to_addr = match Address::from_str(&to) {
        Ok(a) => a,
        Err(_) => return "Error: Failed to parse recipient address.".to_string(),
    };

    // 3) Decode the `data` payload
    let data_bytes = match hex::decode(data.trim_start_matches("0x")) {
        Ok(d) => d,
        Err(_) => return "Error: Failed to decode data field.".to_string(),
    };

    // 4) RLP‐encode the EIP-1559 transaction fields:
    //    [ chain_id, nonce, pri, fee, gas_limit, to, value, data, [] ]
    let mut stream = RlpStream::new_list(9);
    stream.append(&U256::from(self.chain_id));
    stream.append(&U256::from(self.nonce));
    stream.append(&pri);
    stream.append(&fee);
    stream.append(&gas_limit_u256);
    stream.append(&to_addr);
    stream.append(&value_u256);
    stream.append(&data_bytes);
    stream.begin_list(0);
    let rlp_payload = stream.out().to_vec();

    // 5) Compute the pre-signing hash: keccak256(0x02 || rlp_payload)
    let mut hasher = Keccak::v256();
    hasher.update(&[0x02]);
    hasher.update(&rlp_payload);
    let mut sign_hash = [0u8; 32];
    hasher.finalize(&mut sign_hash);

    // 6) Append derivation path bytes so the HW can pick the right key
    let mut to_sign = sign_hash.to_vec();
    match extract_u16s(&self.account_derivation_path) {
        Ok((h1, h2)) => append_integers_as_bytes(&mut to_sign, h1, h2),
        Err(_)      => return "Error: Derivation path error.".to_string(),
    }

    // 7) Return “unsignedRlpHex:&base64(sign_hash||derivation)”
    let unsigned_hex = hex::encode(&rlp_payload);
    let b64        = base64::encode(&to_sign);
    format!("{}:&{}", unsigned_hex, b64)
}
    pub fn prepare_permit2(&self, full_calldata: String) -> String {
        // Debug: raw input
        println!("DEBUG: full_calldata = {}", full_calldata);
        // 1) Decode hex
        let data_hex = full_calldata.trim_start_matches("0x");
        let all = match hex::decode(data_hex) {
            Ok(v) => v,
            Err(e) => {
                println!("DEBUG: decode error: {:?}", e);
                return "Error: invalid calldata hex".to_string();
            }
        };
        println!("DEBUG: decoded bytes len = {}", all.len());
        if all.len() < 4 {
            println!("DEBUG: calldata too short");
            return "Error: calldata too short".to_string();
        }
        // Permit2 selector
        let permit_selector = [0xd5, 0x05, 0xac, 0xcf];
        println!("DEBUG: head4 = {:02x?}", &all[0..4]);

                        // 2) Extract permit_full
        let permit_full: Vec<u8> = if all[0..4] == permit_selector {
            println!("DEBUG: standalone permitTransferFrom");
            all.clone()
        } else {
            println!("DEBUG: nested execute flow");
            // strip execute selector
            let data = &all[4..];
            println!("DEBUG: execute data len = {}", data.len());
            // parse inputs[] dynamic array at head[1]
            if data.len() < 64 {
                println!("DEBUG: missing execute heads");
                return "Error: missing execute head".to_string();
            }
            // head[1] = offset to inputs[] array
            let inputs_off = U256::from_big_endian(&data[32..64]).low_u64() as usize;
            println!("DEBUG: inputs_off = {}", inputs_off);
            if data.len() < inputs_off + 32 {
                println!("DEBUG: invalid inputs offset");
                return "Error: invalid inputs offset".to_string();
            }
            // number of elements in inputs[]
            let inputs_len = U256::from_big_endian(&data[inputs_off..inputs_off+32]).low_u64() as usize;
            println!("DEBUG: inputs_len = {}", inputs_len);
            if inputs_len == 0 {
                println!("DEBUG: no inputs");
                return "Error: no inputs".to_string();
            }
            // scan each inputs element for permit selector
            let mut found = None;
            for i in 0..inputs_len {
                let head = inputs_off + 32 + i * 32;
                println!("DEBUG: input[{}] head at {}", i, head);
                if data.len() < head + 32 {
                    println!("DEBUG: invalid input head");
                    return "Error: invalid input head".to_string();
                }
                // offset to element data
                let rel = U256::from_big_endian(&data[head..head+32]).low_u64() as usize;
                println!("DEBUG: input[{}] rel_off = {}", i, rel);
                if data.len() < rel + 32 {
                    println!("DEBUG: invalid input rel_off");
                    return "Error: invalid input rel_off".to_string();
                }
                // length of element data
                let elem_len = U256::from_big_endian(&data[rel..rel+32]).low_u64() as usize;
                println!("DEBUG: input[{}] elem_len = {}", i, elem_len);
                let start = rel + 32;
                let end = start + elem_len;
                if data.len() < end {
                    println!("DEBUG: invalid input length");
                    return "Error: invalid input length".to_string();
                }
                println!("DEBUG: input[{}] first4 = {:02x?}", i, &data[start..start+4]);
                if elem_len >= 4 && data[start..start+4] == permit_selector {
                    println!("DEBUG: found permit in input[{}]", i);
                    found = Some(data[start..end].to_vec());
                    break;
                }
            }
            match found {
                Some(v) => v,
                None => {
                    println!("DEBUG: permitTransferFrom not found in inputs[]");
                    return ":&".to_string();
                }
            }
        };

        // 3) Compute struct_start offset offset offset
        let struct_start = 4 + 2 * 32;
        println!("DEBUG: struct_start = {}, need {}", struct_start, struct_start + 6 * 32);
        if permit_full.len() < struct_start + 6 * 32 {
            println!("DEBUG: incomplete struct, len {} < {}", permit_full.len(), struct_start + 6 * 32);
            return "Error: incomplete struct".to_string();
        }

        // 4) Build EIP-712 struct hash
        let type_hash = Keccak256::digest(
            b"PermitSingle(address token,address owner,address spender,uint256 amount,uint256 expiration,uint256 nonce)"
        );
        let mut enc = type_hash.to_vec();
        for i in 0..6 {
            let start = struct_start + i * 32;
            let end = start + 32;
            enc.extend(&permit_full[start..end]);
        }
        let struct_hash = Keccak256::digest(&enc);

        // 5) Domain separator
        let domain_typehash = Keccak256::digest(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        );
        let name_hash = Keccak256::digest(b"Permit2");
        let version_hash = Keccak256::digest(b"1");
        let mut domain_enc = domain_typehash.to_vec();
        domain_enc.extend(&name_hash);
        domain_enc.extend(&version_hash);
        let mut chain_bytes = [0u8; 32]; U256::from(self.chain_id).to_big_endian(&mut chain_bytes);
        domain_enc.extend(&chain_bytes);
        let permit2_addr = Address::from_str("<PERMIT2_ADDRESS>").unwrap();
        let mut addr_bytes = [0u8; 32]; addr_bytes[12..].copy_from_slice(&permit2_addr.0);
        domain_enc.extend(&addr_bytes);
        let domain_sep = Keccak256::digest(&domain_enc);

        // 6) Final digest
        let mut digest_data = Vec::with_capacity(2 + 32 + 32);
        digest_data.push(0x19);
        digest_data.push(0x01);
        digest_data.extend(&domain_sep);
        digest_data.extend(&struct_hash);
        let digest = Keccak256::digest(&digest_data);

        // 7) Append derivation path
        let mut to_sign = digest.to_vec();
        if let Ok((h1, h2)) = extract_u16s(&self.account_derivation_path) {
            append_integers_as_bytes(&mut to_sign, h1, h2);
        } else {
            println!("DEBUG: derivation failed");
            return "Error: derivation".to_string();
        }

        // 8) Return raw permit and signing blob
        let hex_no_sig = hex::encode(&permit_full);
        let b64 = base64::encode(&to_sign);
        println!("DEBUG: result= {}:&{}", hex_no_sig, b64);
        format!("{}:&{}", hex_no_sig, b64)
    }

    /// Prepare the full EIP-1559 execute transaction including the signed Permit2.
    /// Returns `<rlp_payload_hex>:&<base64(tx_digest||derivation)>`.
    #[wasm_bindgen(js_name = prepareEip1559Execute)]
    pub fn prepare_eip1559_execute(
        &self,
        to: String,
        value: String,
        max_priority_fee_per_gas: String,
        max_fee_per_gas: String,
        gas_limit: String,
        full_calldata: String,
        permit_signature_b64: String,
    ) -> String {
        // 1. Decode calldata
        let data = match hex::decode(full_calldata.trim_start_matches("0x")) {
            Ok(b) => b,
            Err(_) => return "Error: invalid calldata hex".to_string(),
        };
        // 2. ABI parse & decode execute
        let mut parser = AbiParser::default();
        let execute_fn = match parser.parse_function("execute(bytes,bytes[],address)") {
            Ok(f) => f,
            Err(_) => return "Error: parse execute ABI".to_string(),
        };
        let tokens = match execute_fn.decode_input(&data) {
            Ok(t) => t,
            Err(_) => return "Error: decode execute input".to_string(),
        };
        // 3. Replace inputs[0] with the signed permit
        let mut inputs = match &tokens[1] {
            Token::Array(arr) => arr.clone(),
            _ => return "Error: invalid inputs".to_string(),
        };
        let sig_bytes = match base64::decode(&permit_signature_b64) {
            Ok(b) => b,
            Err(_) => return "Error: invalid permit signature".to_string(),
        };
        let sig_token = Token::Bytes(sig_bytes);
        let permit_fn = match parser.parse_function(
            "permitTransferFrom((address token,address owner,address spender,uint256 amount,uint256 expiration,uint256 nonce),bytes)"
        ) {
            Ok(f) => f,
            Err(_) => return "Error: parse permitTransferFrom ABI".to_string(),
        };
        // Decode original struct to reattach signature
        let original_args = match &tokens[1] {
            Token::Array(arr) => {
                if let Token::Bytes(b) = &arr[0] {
                    match permit_fn.decode_input(b) {
                        Ok(decoded) => match &decoded[0] {
                            Token::Tuple(v) => v.clone(),
                            _ => return "Error: invalid original permit".to_string(),
                        },
                        Err(_) => return "Error: decode original permit".to_string(),
                    }
                } else { return "Error: invalid original permit".to_string() }
            }
            _ => return "Error: invalid inputs".to_string(),
        };
        let new_permit = match permit_fn.encode_input(&[
            Token::Tuple(original_args), sig_token
        ]) {
            Ok(v) => v,
            Err(_) => return "Error: encode new permit".to_string(),
        };
        inputs[0] = Token::Bytes(new_permit);
        // 4. Re-encode full calldata
        let recipient = tokens[2].clone();
        let new_calldata = match execute_fn.encode_input(&[
            tokens[0].clone(), Token::Array(inputs), recipient
        ]) {
            Ok(v) => v,
            Err(_) => return "Error: re-encode execute calldata".to_string(),
        };
        // 5. Build EIP-1559 RLP
        let to_addr = Address::from_str(&to).unwrap_or_default();
        let val = U256::from_dec_str(&value).unwrap_or_default();
        let pri = U256::from_dec_str(&max_priority_fee_per_gas).unwrap_or_default();
        let fee = U256::from_dec_str(&max_fee_per_gas).unwrap_or_default();
        let gas_lim = U256::from_dec_str(&gas_limit).unwrap_or_default();
        let mut stream = RlpStream::new_list(9);
        stream.append(&U256::from(self.chain_id));
        stream.append(&U256::from(self.nonce));
        stream.append(&pri);
        stream.append(&fee);
        stream.append(&gas_lim);
        stream.append(&to_addr);
        stream.append(&val);
        stream.append(&new_calldata);
        stream.begin_list(0);
        let rlp_payload = stream.out().to_vec();
        // 6. Compute transaction digest
        let mut hasher = Keccak256::new();
        hasher.update(&[0x02]);
        hasher.update(&rlp_payload);
        let tx_hash = hasher.finalize();
        // 7. Append derivation and return
        let mut to_sign = tx_hash.to_vec();
        if let Ok((a,b)) = extract_u16s(&self.account_derivation_path) {
            append_integers_as_bytes(&mut to_sign, a, b);
        } else {
            return "Error: derivation".to_string();
        }
        let hex_rlp = hex::encode(rlp_payload);
        let sig_blob = base64::encode(&to_sign);
        format!("{}:&{}", hex_rlp, sig_blob)
    }
    /// Reconstruct & broadcast a signed EIP-1559 tx from `<hex-rlp>` + base64 signature.
    pub async fn broadcast_eip1559(&mut self, unsigned_tx: String, tx_signature: String) -> String {
        // 1) decode the RLP payload
        let hex_str = unsigned_tx.trim_start_matches("0x");
        let mut raw = match hex::decode(hex_str) {
            Ok(b) => b,
            Err(_) => return "Error: Failed to decode the unsigned transaction.".to_string(),
        };
        // if it starts with the type‐2 marker, strip it off:
        if raw.first() == Some(&0x02) {
            raw = raw[1..].to_vec();
        }
        let rlp = Rlp::new(&raw);

        // 2) extract each field with explicit matches
        let chain_id = match rlp.at(0).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode chain ID.".to_string(),
        };
        let nonce = match rlp.at(1).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode nonce.".to_string(),
        };
        let max_prio = match rlp.at(2).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode max priority fee.".to_string(),
        };
        let max_fee = match rlp.at(3).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode max fee.".to_string(),
        };
        let gas_limit = match rlp.at(4).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode gas limit.".to_string(),
        };
        let to_addr = match rlp.at(5).and_then(|f| f.data().map(|d| d.to_vec())) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode to address.".to_string(),
        };
        let value = match rlp.at(6).and_then(|f| f.as_val::<U256>()) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode value.".to_string(),
        };
        let data_field = match rlp.at(7).and_then(|f| f.data().map(|d| d.to_vec())) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode data field.".to_string(),
        };
        // 8th element is the accessList — we know it was empty, so skip explicit decode

        // 3) decode the signature blob
        let sig = match base64::decode(&tx_signature) {
            Ok(b) => b,
            Err(_) => return "Error: Failed to decode the transaction signature.".to_string(),
        };
        if sig.len() < 65 {
            return "Error: Signature length is invalid.".to_string();
        }
        let r_sig = &sig[0..32];
        let s_sig = &sig[32..64];
        let v_raw = sig[64];
        let rec_id = if v_raw > 1 { v_raw - 27 } else { v_raw };

        // 4) RLP-encode the signed tx:
        let mut stre = RlpStream::new_list(12);
        stre.append(&chain_id);
        stre.append(&nonce);
        stre.append(&max_prio);
        stre.append(&max_fee);
        stre.append(&gas_limit);
        stre.append(&to_addr);
        stre.append(&value);
        stre.append(&data_field);
        stre.begin_list(0);               // empty accessList
        stre.append(&U256::from(rec_id));
        stre.append(&r_sig);
        stre.append(&s_sig);
        let signed_rlp = stre.out().to_vec();

        // 5) prepend type byte and hex
        let mut raw = Vec::with_capacity(signed_rlp.len() + 1);
        raw.push(0x02);
        raw.extend(&signed_rlp);
        let raw_hex = format!("0x{}", hex::encode(raw));

        //let raw_hex = format!("0x{}", hex::encode(raw_tx));
        // 👉 DEBUG: print (or even return) the fully signed RLP so you can inspect it
        //println!("DEBUG signed_raw_tx: {}", raw_hex);
        //return raw_hex;   // <— you can early‐return here for testing
        // 6) broadcast
        let client = Client::new();
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [ raw_hex ],
            "id": 1
        });
        if let Ok(resp) = client.post(&self.infura_url).json(&body).send().await {
            if let Ok(j) = resp.json::<serde_json::Value>().await {
                if let Some(r) = j.get("result").and_then(|r| r.as_str()) {
                    return r.to_string();
                }
                if let Some(e) = j.get("error") {
                    return format!("Error: {:?}", e);
                }
            }
        }
        "Error: Failed to broadcast transaction.".to_string()
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
    pub fn construct_signed_tx(&self, unsigned_tx: String, tx_signature: String) -> String {
        // 1. strip 0x and hex-decode
        let unsigned_hex = unsigned_tx.trim_start_matches("0x");
        let mut tx_bytes = match hex::decode(unsigned_hex) {
            Ok(b) => b,
            Err(_) => return "Error: Failed to decode the unsigned transaction.".to_string(),
        };

        // 2. detect & strip EIP-1559 prefix
        let is_eip1559 = tx_bytes.get(0).map(|b| *b == 0x02).unwrap_or(false);
        if is_eip1559 {
            // drop the 0x02 tag
            tx_bytes = tx_bytes.split_off(1);
        }

        // 3. RLP-decode
        let rlp = Rlp::new(&tx_bytes);

        // 4. helpers for per-field error handling
        let get_u256 = |idx: usize, msg: &str| {
            rlp.at(idx)
                .and_then(|f| f.as_val::<U256>())
                .map_err(|_| msg.to_string())
        };
        let get_bytes = |idx: usize, msg: &str| {
            rlp.at(idx)
                .and_then(|f| f.data())
                .map(|d| d.to_vec())
                .map_err(|_| msg.to_string())
        };

        // 5. pull out fields in the correct order
        let (chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data_field) =
            if is_eip1559 {
                // type-2 fields: [chainId, nonce, maxPriorityFeePerGas,
                //                   maxFeePerGas, gasLimit, to, value, data, accessList…]
                let chain_id = match get_u256(0, "Error: Failed to decode the chain ID.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let nonce = match get_u256(1, "Error: Failed to decode the nonce.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let max_priority_fee = match get_u256(
                    2,
                    "Error: Failed to decode the max priority fee per gas.",
                ) {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let max_fee = match get_u256(3, "Error: Failed to decode the max fee per gas.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let gas_limit = match get_u256(4, "Error: Failed to decode the gas limit.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let to = match get_bytes(5, "Error: Failed to decode the output.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let value = match get_u256(6, "Error: Failed to decode the value.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let data_field = match get_bytes(7, "Error: Failed to decode the data field.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                (chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data_field)
            } else {
                // legacy fields: [nonce, gasPrice, gasLimit, to, value, data, chainId…]
                let nonce = match get_u256(0, "Error: Failed to decode the nonce.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let gas_price = match get_u256(1, "Error: Failed to decode the gas price.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let gas_limit = match get_u256(2, "Error: Failed to decode the gas limit.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let to = match get_bytes(3, "Error: Failed to decode the output.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let value = match get_u256(4, "Error: Failed to decode the value.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let data_field = match get_bytes(5, "Error: Failed to decode the data field.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                let chain_id = match get_u256(6, "Error: Failed to decode the chain ID.") {
                    Ok(v) => v,
                    Err(e) => return e,
                };
                // for legacy, both priority & max fees are simply gasPrice
                (chain_id, nonce, gas_price, gas_price, gas_limit, to, value, data_field)
            };

        // 6. decode the signature
        let sig_bytes = match base64::decode(&tx_signature) {
            Ok(v) => v,
            Err(_) => return "Error: Failed to decode the transaction signature.".to_string(),
        };
        if sig_bytes.len() < 65 {
            return "Error: Failed to decode the transaction signature.".to_string();
        }
        let r_sig = &sig_bytes[0..32];
        let s_sig = &sig_bytes[32..64];
        let v_raw = sig_bytes[64];
        let recovery_id = if v_raw > 1 { v_raw - 27 } else { v_raw };
        let v_calc = chain_id.low_u64() * 2 + 35 + recovery_id as u64;

        let v_field = if is_eip1559 {
            U256::from(recovery_id as u64)
        } else {
            U256::from(chain_id.low_u64() * 2 + 35 + recovery_id as u64)
        };

        // 7. rebuild the signed RLP
        let mut stream = if is_eip1559 {
            RlpStream::new_list(12)
        } else {
            RlpStream::new_list(9)
        };

        if is_eip1559 {
            stream.append(&chain_id);
            stream.append(&nonce);
            stream.append(&max_priority_fee);
            stream.append(&max_fee);
            stream.append(&gas_limit);
            stream.append(&to);
            stream.append(&value);
            stream.append(&data_field);
            stream.begin_list(0); // empty accessList
            stream.append_raw(&[recovery_id], 1);
            stream.append(&r_sig);
            stream.append(&s_sig);
        } else {
            stream.append(&nonce);
            stream.append(&max_priority_fee); // gasPrice
            stream.append(&gas_limit);
            stream.append(&to);
            stream.append(&value);
            stream.append(&data_field);
            stream.append(&v_calc);
            stream.append(&r_sig);
            stream.append(&s_sig);
        }

        let mut signed_bytes = stream.out().to_vec();
        if is_eip1559 {
            // re-prefix with 0x02
            let mut pref = vec![0x02];
            pref.append(&mut signed_bytes);
            signed_bytes = pref;
        }

        format!("0x{}", hex::encode(&signed_bytes))
    }
    pub fn hex_to_b64(&self, tx_hash : String) -> String{
        let hex_str = tx_hash.strip_prefix("0x").unwrap_or(&tx_hash);
        let bytes = match hex::decode(hex_str){
            Ok(hex) => hex,
            Err(_) => return "Error: Invalid hex string.".to_string(),
        };
        return base64::encode(&bytes);
    }
    pub fn get_nonce(&self) -> u64{
        return self.nonce;
    }
    pub fn get_chain_id(&self) -> u64{
        return self.chain_id;
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

