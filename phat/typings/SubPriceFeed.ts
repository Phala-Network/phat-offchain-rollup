import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "@devphase/service";
import type * as DPT from "@devphase/service/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace SubPriceFeed {
    type InkPrimitives_Types_AccountId$1 = any;
    type InkPrimitives_LangError$4 = {
        CouldNotReadInput? : null
        };
    type Result$2 = {
        Ok? : never[],
        Err? : InkPrimitives_LangError$4
        };
    type Result$5 = {
        Ok? : InkPrimitives_Types_AccountId$1,
        Err? : InkPrimitives_LangError$4
        };
    type SubPriceFeed_SubPriceFeed_Error$8 = {
        BadOrigin? : null,
        NotConfigured? : null,
        InvalidKeyLength? : null,
        FailedToCreateClient? : null,
        FailedToCommitTx? : null,
        FailedToFetchPrice? : null,
        FailedToGetNameOwner? : null,
        FailedToClaimName? : null,
        FailedToGetStorage? : null,
        FailedToCreateTransaction? : null,
        FailedToSendTransaction? : null,
        FailedToGetBlockHash? : null,
        FailedToDecode? : null,
        RollupAlreadyInitialized? : null,
        RollupConfiguredByAnotherAccount? : null
        };
    type Result$7 = {
        Ok? : never[],
        Err? : SubPriceFeed_SubPriceFeed_Error$8
        };
    type Result$6 = {
        Ok? : Result$7,
        Err? : InkPrimitives_LangError$4
        };
    type Option$11 = {
        None? : null,
        Some? : number[] | string
        };
    type Result$10 = {
        Ok? : Option$11,
        Err? : SubPriceFeed_SubPriceFeed_Error$8
        };
    type Result$9 = {
        Ok? : Result$10,
        Err? : InkPrimitives_LangError$4
        };
    type InkPrimitives_Types_Hash$12 = any;
    type PinkExtension_ChainExtension_PinkExt$13 = {

        };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$5>>>;
        }

        export interface MaybeInitRollup extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$9>>>;
        }

        export interface FeedPrice extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$9>>>;
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
            (options: ContractOptions, rpc: string, pallet_id: number, submit_key: number[] | string, token0: string, token1: string): DPT.SubmittableExtrinsic;
        }

        export interface TransferOwnership extends DPT.ContractTx {
            (options: ContractOptions, new_owner: InkPrimitives_Types_AccountId$1): DPT.SubmittableExtrinsic;
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
