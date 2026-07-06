/**
 * Stellar SDK utilities and contract interactions
 */

import * as StellarSdk from '@stellar/stellar-sdk';

// Network configuration
export const NETWORK_PASSPHRASE = process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || 
    StellarSdk.Networks.TESTNET;

export const HORIZON_URL = process.env.NEXT_PUBLIC_HORIZON_URL || 
    'https://horizon-testnet.stellar.org';

export const SOROBAN_RPC_URL = process.env.NEXT_PUBLIC_SOROBAN_RPC_URL || 
    'https://soroban-testnet.stellar.org';

export const CONTRACT_ID = process.env.NEXT_PUBLIC_CONTRACT_ID || '';

export const NATIVE_TOKEN = process.env.NEXT_PUBLIC_NATIVE_TOKEN || 
    'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC';

// Initialize clients
export const server = new StellarSdk.Horizon.Server(HORIZON_URL);
export const sorobanServer = new StellarSdk.SorobanRpc.Server(SOROBAN_RPC_URL);

/**
 * Convert stroops to XLM
 */
export function stroopsToXLM(stroops: bigint | number): number {
    return Number(stroops) / 10_000_000;
}

/**
 * Convert XLM to stroops
 */
export function xlmToStroops(xlm: number): bigint {
    return BigInt(Math.floor(xlm * 10_000_000));
}

/**
 * Format XLM amount for display
 */
export function formatXLM(stroops: bigint | number, decimals: number = 2): string {
    const xlm = stroopsToXLM(stroops);
    return new Intl.NumberFormat('en-US', {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
    }).format(xlm);
}

/**
 * Format token amount for display
 */
export function formatTokenAmount(amount: bigint | number, decimals: number = 2): string {
    const value = Number(amount) / 10_000_000;
    return new Intl.NumberFormat('en-US', {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
    }).format(value);
}

/**
 * Shorten address for display
 */
export function shortenAddress(address: string, chars: number = 4): string {
    return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

/**
 * Build and simulate a transaction
 */
export async function buildAndSimulateTransaction(
    sourceAccount: string,
    operation: StellarSdk.xdr.Operation,
): Promise<StellarSdk.Transaction> {
    const account = await server.loadAccount(sourceAccount);
    
    const transaction = new StellarSdk.TransactionBuilder(account, {
        fee: StellarSdk.BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
    })
        .addOperation(operation)
        .setTimeout(180)
        .build();

    // Simulate to get proper auth and resource fees
    const simulated = await sorobanServer.simulateTransaction(transaction);
    
    if (StellarSdk.SorobanRpc.Api.isSimulationError(simulated)) {
        throw new Error(`Simulation failed: ${simulated.error}`);
    }

    return StellarSdk.SorobanRpc.assembleTransaction(
        transaction,
        simulated,
    ).build();
}

/**
 * Get contract instance
 */
export function getContract(contractId: string = CONTRACT_ID): StellarSdk.Contract {
    return new StellarSdk.Contract(contractId);
}

/**
 * Parse contract error
 */
export function parseContractError(error: any): string {
    if (error?.message) {
        // Try to extract error code from message
        const match = error.message.match(/Error\(Contract, #(\d+)\)/);
        if (match) {
            const errorCode = parseInt(match[1]);
            return getErrorMessage(errorCode);
        }
        return error.message;
    }
    return 'An unknown error occurred';
}

/**
 * Map error codes to human-readable messages
 */
function getErrorMessage(code: number): string {
    const errorMessages: Record<number, string> = {
        1: 'Contract already initialized',
        2: 'Contract not initialized',
        10: 'Invalid amount',
        11: 'Invalid fee',
        12: 'Invalid name',
        13: 'Invalid symbol',
        30: 'Launch not found',
        31: 'Launch not active',
        32: 'Target not reached yet',
        33: 'Already migrated',
        34: 'Insufficient supply remaining',
        50: 'Slippage tolerance exceeded',
        51: 'Insufficient balance',
        70: 'Math overflow',
        71: 'Division by zero',
        80: 'Unauthorized access',
    };
    
    return errorMessages[code] || `Contract error #${code}`;
}

/**
 * Check if Freighter wallet is installed
 */
export async function isFreighterInstalled(): Promise<boolean> {
    if (typeof window === 'undefined') return false;
    
    return new Promise((resolve) => {
        const checkFreighter = () => {
            if ((window as any).freighter) {
                resolve(true);
            } else {
                resolve(false);
            }
        };
        
        if (document.readyState === 'complete') {
            checkFreighter();
        } else {
            window.addEventListener('load', checkFreighter);
        }
    });
}

/**
 * Request access to Freighter wallet
 */
export async function connectFreighter(): Promise<string> {
    const freighter = (window as any).freighter;
    
    if (!freighter) {
        throw new Error('Freighter wallet not installed');
    }
    
    const publicKey = await freighter.requestAccess();
    
    if (!publicKey) {
        throw new Error('Wallet access denied');
    }
    
    return publicKey;
}

/**
 * Sign and submit a transaction with Freighter
 */
export async function signAndSubmitWithFreighter(
    transaction: StellarSdk.Transaction,
): Promise<StellarSdk.SorobanRpc.Api.GetTransactionResponse> {
    const freighter = (window as any).freighter;
    
    if (!freighter) {
        throw new Error('Freighter wallet not installed');
    }
    
    const xdr = transaction.toXDR();
    const signedXDR = await freighter.signTransaction(xdr, {
        network: process.env.NEXT_PUBLIC_NETWORK || 'TESTNET',
        networkPassphrase: NETWORK_PASSPHRASE,
    });
    
    const signedTx = StellarSdk.TransactionBuilder.fromXDR(
        signedXDR,
        NETWORK_PASSPHRASE,
    ) as StellarSdk.Transaction;
    
    const response = await sorobanServer.sendTransaction(signedTx);
    
    if (response.status === 'ERROR') {
        throw new Error('Transaction submission failed');
    }
    
    // Poll for result
    let getResponse = await sorobanServer.getTransaction(response.hash);
    let attempts = 0;
    
    while (getResponse.status === StellarSdk.SorobanRpc.Api.GetTransactionStatus.NOT_FOUND && attempts < 30) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        getResponse = await sorobanServer.getTransaction(response.hash);
        attempts++;
    }
    
    if (getResponse.status === StellarSdk.SorobanRpc.Api.GetTransactionStatus.FAILED) {
        throw new Error('Transaction failed');
    }
    
    return getResponse;
}
