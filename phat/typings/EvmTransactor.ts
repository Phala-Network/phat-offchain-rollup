import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace EvmTransactor {
    type InkEnv_Types_AccountId = any;
    type PrimitiveTypes_H160 = any;

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.ICompact<InkEnv_Types_AccountId>>>;
        }

        export interface Wallet extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.ICompact<PrimitiveTypes_H160>>>;
        }

        export interface GetRetiredSecretKey extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<any>>;
        }

        export interface Poll extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<any>>;
        }

        export interface TestPollWithKey extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, key: DPT.FixedArray<number, 32>): DPT.CallResult<DPT.CallOutcome<any>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        owner: ContractQuery.Owner;
        wallet: ContractQuery.Wallet;
        getRetiredSecretKey: ContractQuery.GetRetiredSecretKey;
        poll: ContractQuery.Poll;
        testPollWithKey: ContractQuery.TestPollWithKey;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, rollup_handler: InkEnv_Types_AccountId, anchor: PrimitiveTypes_H160): DPT.SubmittableExtrinsic;
        }

        export interface RetireWallet extends DPT.ContractTx {
            (options: ContractOptions): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
        config: ContractTx.Config;
        retireWallet: ContractTx.RetireWallet;
    }

    /** */
    /** Contract */
    /** */
    export declare class Contract extends DPT.Contract {
        get query(): MapMessageQuery;
        get tx(): MapMessageTx;
    }

    /** */
    /** Contract factory */
    /** */
    export declare class Factory extends DevPhase.ContractFactory {
        instantiate<T = Contract>(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<T>;
    }
}
