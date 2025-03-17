/* tslint:disable */
/* eslint-disable */
export class Wallet {
  free(): void;
  constructor(xpub: string, infura_url: string, chain_id: bigint);
  sync(): Promise<string>;
  send(to: string, value: string, fee_rate: number): string;
  broadcast(unsigned_tx: string, tx_signature: string): Promise<string>;
  address(): string;
  balance(): string;
  estimate_fee(fee_rate: number): string;
  nonce(): bigint;
}
