import Web3 from 'web3';

import { ABI, CONTRACT_ADDRESS } from './contracts/Marketplace';
import { ChainId } from '../components/MetamaskConnect';

export default class MarketplaceClient {
  private address: string;
  private web3: Web3;
  private chainId: ChainId;

  constructor(address: string, ethereum: any, chainId: ChainId) {
    this.address = address;
    this.web3 = new Web3(ethereum);
    this.chainId = chainId;
  }

  async transferOwnership(newAddress: string) {
    const contract = this.getContract();
    return contract.methods
      .transferOwnership(newAddress)
      .send({ from: this.address });
  }

  async adminSetInterestRate(rate: number) {
    const contract = this.getContract();
    return contract.methods
      .adminSetInterestRate(rate)
      .send({ from: this.address });
  }

  async adminWithdraw(amount: bigint) {
    const contract = this.getContract();
    return contract.methods.adminWithdraw(amount).send({ from: this.address });
  }

  async liquidityWithdrawable(): Promise<bigint> {
    const contract = this.getContract();
    return contract.methods.liquidityWithdrawable().call();
  }

  async adminSetRewardPool(newAddress: string) {
    const contract = this.getContract();
    return contract.methods
      .adminSetRewardPool(newAddress)
      .send({ from: this.address });
  }

  async tokenPriceForCaller(contractId: bigint): Promise<bigint> {
    const contract = this.getContract();
    return contract.methods.tokenPriceForCaller(contractId).call();
  }

  async interestRate(): Promise<bigint> {
    const contract = this.getContract();
    return contract.methods.interestRate().call();
  }

  async usdErc20(): Promise<string> {
    const contract = this.getContract();
    return contract.methods.usdErc20().call();
  }

  async rewardPool(): Promise<string> {
    const contract = this.getContract();
    return contract.methods.rewardPool().call();
  }

  private getContract() {
    return new this.web3.eth.Contract(ABI, CONTRACT_ADDRESS[this.chainId]);
  }
}
