/* tslint:disable */
/* eslint-disable */
export class Wallet {
  free(): void;
  constructor(xpub: string, account_derivation_path: string, infura_url: string, chain_id: bigint);
  sync(): Promise<string>;
  send(to: string, value: string, fee_rate: number): string;
  validate_contract(contract_address: string): Promise<string>;
  erc20_transfer(contract_address: string, recipient: string, token_amount: string, fee_rate: number): string;
  erc20_balance(contract_address: string): Promise<string>;
  broadcast(unsigned_tx: string, tx_signature: string): Promise<string>;
  address(): string;
  balance(): string;
  estimate_fee(fee_rate: number): string;
  nonce(): bigint;
}
