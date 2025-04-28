/* tslint:disable */
/* eslint-disable */
export class Wallet {
  free(): void;
  constructor(xpub: string, account_derivation_path: string, infura_url: string, chain_id: bigint);
  sync(): Promise<string>;
  send(to: string, value: string, fee_rate: number): string;
  send_eip1559(to: string, value: string, fee_rate: number): string;
  prepare_eip1559(to: string, value: string, max_priority_fee_per_gas: string, max_fee_per_gas: string, gas_limit: string, data: string): string;
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
}
