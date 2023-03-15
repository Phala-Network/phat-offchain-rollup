import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace Sub0Factory {
    type InkPrimitives_Types_AccountId = any;
    type InkPrimitives_Types_Hash = any;
    type InkPrimitives_LangError = { CouldNotReadInput: null };
    type Result = { Ok: InkPrimitives_Types_AccountId } | { Err: InkPrimitives_LangError };
    type Sub0Factory_Sub0Factory_Error = { BadOrigin: null } | { NotConfigured: null } | { InvalidKeyLength: null } | { FailedToDeployContract: null } | { FailedToConfigContract: null } | { FailedToTransferOwnership: null };
    type Sub0Factory_Sub0Factory_Deployment = { name: string, owner: InkPrimitives_Types_AccountId, contract_id: InkPrimitives_Types_AccountId, created_at: number, expired_at: number };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetConfig extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface GetDeployments extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        getConfig: ContractQuery.GetConfig;
        getDeployments: ContractQuery.GetDeployments;
        owner: ContractQuery.Owner;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, pallet_id: number, submit_key: number[], price_feed_code: InkPrimitives_Types_Hash): DPT.SubmittableExtrinsic;
        }

        export interface DeployPriceFeed extends DPT.ContractTx {
            (options: ContractOptions, name: string, token0: string, token1: string): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
        config: ContractTx.Config;
        deployPriceFeed: ContractTx.DeployPriceFeed;
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
