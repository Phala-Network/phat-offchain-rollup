import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace SubPriceFeed {
    type InkEnv_Types_AccountId = any;
    type SubPriceFeed_SubPriceFeed_Error = { BadOrigin: null } | { NotConfigured: null } | { InvalidKeyLength: null } | { FailedToCreateClient: null } | { FailedToCommitTx: null } | { FailedToFetchPrice: null } | { FailedToGetNameOwner: null } | { FailedToClaimName: null } | { FailedToGetStorage: null } | { FailedToCreateTransaction: null } | { FailedToSendTransaction: null } | { FailedToGetBlockHash: null } | { FailedToDecode: null } | { RollupAlreadyInitialized: null } | { RollupConfiguredByAnotherAccount: null };
    type Result = { Ok: Option } | { Err: SubPriceFeed_SubPriceFeed_Error };
    type Option = { None: null } | { Some: number[] };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<InkEnv_Types_AccountId>>>;
        }

        export interface MaybeInitRollup extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface FeedPrice extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        owner: ContractQuery.Owner;
        maybeInitRollup: ContractQuery.MaybeInitRollup;
        feedPrice: ContractQuery.FeedPrice;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, pallet_id: number, submit_key: number[], token0: string, token1: string): DPT.SubmittableExtrinsic;
        }

        export interface TransferOwnership extends DPT.ContractTx {
            (options: ContractOptions, new_owner: InkEnv_Types_AccountId): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
        config: ContractTx.Config;
        transferOwnership: ContractTx.TransferOwnership;
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
