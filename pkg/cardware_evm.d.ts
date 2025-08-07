/* tslint:disable */
/* eslint-disable */
export class Wallet {
  free(): void;
  constructor(xpub: string, account_derivation_path: string, infura_url: string, chain_id: bigint);
  sync(): Promise<string>;
  send(to: string, value: string, fee_rate: number): string;
  send_eip1559(to: string, value: string, fee_rate: number): string;
  prepare_eip1559(to: string, value: string, max_priority_fee_per_gas: string, max_fee_per_gas: string, gas_limit: string, data: string): string;
  prepare_sign_typed_data_v4(typed_data_json: string): string;
  signature_hex_from_b64(tx_signature_b64: string): string;
  prepare_personal_sign(message_hex: string): string;
  prepare_eip1559_transfer(to: string, value: string, data: string): string;
  /**
   * Reconstruct & broadcast a signed EIP-1559 tx from `<hex-rlp>` + base64 signature.
   */
  broadcast_eip1559(unsigned_tx: string, tx_signature: string): Promise<string>;
  validate_contract(contract_address: string): Promise<string>;
  erc20_transfer(contract_address: string, recipient: string, token_amount: string, fee_rate: number): string;
  erc20_balance(contract_addresses: string[]): Promise<string[]>;
  broadcast(unsigned_tx: string, tx_signature: string): Promise<string>;
  construct_signed_tx(unsigned_tx: string, tx_signature: string): string;
  hex_to_b64(tx_hash: string): string;
  get_nonce(): bigint;
  get_chain_id(): bigint;
  address(): string;
  balance(): string;
  estimate_fee(fee_rate: number, gas_limit: number): string;
  nonce(): bigint;
  get_tx_history(base_url: string, api_key: string, limit: number): Promise<string>;
  erc721_transfer(contract_address: string, to: string, token_id: string, fee_rate: number): string;
  erc721_balance(contract_addresses: string[]): Promise<string[]>;
  erc721_owner_of(contract_address: string, token_id: string): Promise<string>;
  erc1155_balance_of(contract_address: string, owner: string, token_id: string): Promise<string>;
  erc1155_transfer(contract_address: string, to: string, token_id: string, amount: string, fee_rate: number): string;
}
