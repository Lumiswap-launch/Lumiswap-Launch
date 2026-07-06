/**
 * Lumiswap Launch Contract Client
 * Type-safe wrapper for contract interactions
 */

import * as StellarSdk from '@stellar/stellar-sdk';
import {
    getContract,
    buildAndSimulateTransaction,
    signAndSubmitWithFreighter,
    parseContractError,
    sorobanServer,
} from './stellar';

export interface LaunchConfig {
    token: string;
    name: string;
    symbol: string;
    totalSupply: bigint;
    targetXlm: bigint;
    virtualXlm: bigint;
}

export interface Launch {
    id: bigint;
    creator: string;
    token: string;
    name: string;
    symbol: string;
    totalSupply: bigint;
    sold: bigint;
    xlmRaised: bigint;
    targetXlm: bigint;
    status: LaunchStatus;
    createdAt: bigint;
    migratedAt: bigint;
}

export enum LaunchStatus {
    Active = 0,
    TargetReached = 1,
    Migrated = 2,
}

export interface CurveState {
    virtualXlm: bigint;
    virtualTokens: bigint;
    k: bigint;
}

export class LumiswapContractClient {
    private contract: StellarSdk.Contract;

    constructor(contractId: string) {
        this.contract = getContract(contractId);
    }

    /**
     * Create a new token launch
     */
    async createLaunch(
        creator: string,
        config: LaunchConfig,
    ): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'create_launch',
                StellarSdk.nativeToScVal(creator, { type: 'address' }),
                StellarSdk.nativeToScVal({
                    token: config.token,
                    name: config.name,
                    symbol: config.symbol,
                    total_supply: config.totalSupply,
                    target_xlm: config.targetXlm,
                    virtual_xlm: config.virtualXlm,
                }, { type: 'LaunchConfig' }),
            );

            const tx = await buildAndSimulateTransaction(creator, operation);
            const result = await signAndSubmitWithFreighter(tx);

            if (
                result.status === StellarSdk.SorobanRpc.Api.GetTransactionStatus.SUCCESS &&
                result.returnValue
            ) {
                return StellarSdk.scValToNative(result.returnValue) as bigint;
            }

            throw new Error('Launch creation failed');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Buy tokens with XLM
     */
    async buy(
        buyer: string,
        launchId: bigint,
        xlmAmount: bigint,
        minTokens: bigint = 0n,
    ): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'buy',
                StellarSdk.nativeToScVal(buyer, { type: 'address' }),
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
                StellarSdk.nativeToScVal(xlmAmount, { type: 'i128' }),
                StellarSdk.nativeToScVal(minTokens, { type: 'i128' }),
            );

            const tx = await buildAndSimulateTransaction(buyer, operation);
            const result = await signAndSubmitWithFreighter(tx);

            if (
                result.status === StellarSdk.SorobanRpc.Api.GetTransactionStatus.SUCCESS &&
                result.returnValue
            ) {
                return StellarSdk.scValToNative(result.returnValue) as bigint;
            }

            throw new Error('Buy failed');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Sell tokens for XLM
     */
    async sell(
        seller: string,
        launchId: bigint,
        tokenAmount: bigint,
        minXlm: bigint = 0n,
    ): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'sell',
                StellarSdk.nativeToScVal(seller, { type: 'address' }),
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
                StellarSdk.nativeToScVal(tokenAmount, { type: 'i128' }),
                StellarSdk.nativeToScVal(minXlm, { type: 'i128' }),
            );

            const tx = await buildAndSimulateTransaction(seller, operation);
            const result = await signAndSubmitWithFreighter(tx);

            if (
                result.status === StellarSdk.SorobanRpc.Api.GetTransactionStatus.SUCCESS &&
                result.returnValue
            ) {
                return StellarSdk.scValToNative(result.returnValue) as bigint;
            }

            throw new Error('Sell failed');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Migrate launch to DEX
     */
    async migrate(caller: string, launchId: bigint): Promise<void> {
        try {
            const operation = this.contract.call(
                'migrate',
                StellarSdk.nativeToScVal(caller, { type: 'address' }),
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
            );

            const tx = await buildAndSimulateTransaction(caller, operation);
            const result = await signAndSubmitWithFreighter(tx);

            if (result.status !== StellarSdk.SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
                throw new Error('Migration failed');
            }
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get launch details
     */
    async getLaunch(launchId: bigint): Promise<Launch> {
        try {
            const operation = this.contract.call(
                'get_launch',
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
            );

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as Launch;
            }

            throw new Error('Failed to fetch launch');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get bonding curve state
     */
    async getCurve(launchId: bigint): Promise<CurveState> {
        try {
            const operation = this.contract.call(
                'get_curve',
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
            );

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as CurveState;
            }

            throw new Error('Failed to fetch curve');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get current price
     */
    async getCurrentPrice(launchId: bigint): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'get_current_price',
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
            );

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as bigint;
            }

            throw new Error('Failed to fetch price');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get buy quote
     */
    async getBuyQuote(launchId: bigint, xlmAmount: bigint): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'get_buy_quote',
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
                StellarSdk.nativeToScVal(xlmAmount, { type: 'i128' }),
            );

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as bigint;
            }

            throw new Error('Failed to get quote');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get sell quote
     */
    async getSellQuote(launchId: bigint, tokenAmount: bigint): Promise<bigint> {
        try {
            const operation = this.contract.call(
                'get_sell_quote',
                StellarSdk.nativeToScVal(launchId, { type: 'u64' }),
                StellarSdk.nativeToScVal(tokenAmount, { type: 'i128' }),
            );

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as bigint;
            }

            throw new Error('Failed to get quote');
        } catch (error) {
            throw new Error(parseContractError(error));
        }
    }

    /**
     * Get launch count
     */
    async getLaunchCount(): Promise<bigint> {
        try {
            const operation = this.contract.call('get_launch_count');

            const account = await sorobanServer.getAccount(this.contract.address());
            const tx = new StellarSdk.TransactionBuilder(account, {
                fee: StellarSdk.BASE_FEE,
                networkPassphrase: StellarSdk.Networks.TESTNET,
            })
                .addOperation(operation)
                .setTimeout(30)
                .build();

            const simulated = await sorobanServer.simulateTransaction(tx);

            if (
                StellarSdk.SorobanRpc.Api.isSimulationSuccess(simulated) &&
                simulated.result
            ) {
                const result = simulated.result.retval;
                return StellarSdk.scValToNative(result) as bigint;
            }

            return 0n;
        } catch (error) {
            return 0n;
        }
    }
}
