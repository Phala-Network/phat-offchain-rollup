import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "@devphase/service";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { ContractExecResult } from "@polkadot/types/interfaces/contracts";
import type * as DPT from "@devphase/service/etc/typings";
import type * as PT from "@polkadot/types";
import type * as PTI from "@polkadot/types/interfaces";
import type * as PTT from "@polkadot/types/types";


/** */
/** Exported types */
/** */

export namespace InkPrimitives {
    export interface LangError {
        couldNotReadInput?: null;
    }

    export namespace LangError$ {
        export enum Enum {
            CouldNotReadInput = "CouldNotReadInput"
        }

        export type Human = InkPrimitives.LangError$.Enum.CouldNotReadInput;
        export type Codec = DPT.Enum<InkPrimitives.LangError$.Enum.CouldNotReadInput, never, never, PTT.Codec>;
    }
}

export namespace SubPriceFeed {
    export interface Error {
        badOrigin?: null;
        notConfigured?: null;
        invalidKeyLength?: null;
        failedToCreateClient?: null;
        failedToCommitTx?: null;
        failedToFetchPrice?: null;
        failedToGetNameOwner?: null;
        failedToClaimName?: null;
        failedToGetStorage?: null;
        failedToCreateTransaction?: null;
        failedToSendTransaction?: null;
        failedToGetBlockHash?: null;
        failedToDecode?: null;
        rollupAlreadyInitialized?: null;
        rollupConfiguredByAnotherAccount?: null;
    }

    export namespace Error$ {
        export enum Enum {
            BadOrigin = "BadOrigin",
            NotConfigured = "NotConfigured",
            InvalidKeyLength = "InvalidKeyLength",
            FailedToCreateClient = "FailedToCreateClient",
            FailedToCommitTx = "FailedToCommitTx",
            FailedToFetchPrice = "FailedToFetchPrice",
            FailedToGetNameOwner = "FailedToGetNameOwner",
            FailedToClaimName = "FailedToClaimName",
            FailedToGetStorage = "FailedToGetStorage",
            FailedToCreateTransaction = "FailedToCreateTransaction",
            FailedToSendTransaction = "FailedToSendTransaction",
            FailedToGetBlockHash = "FailedToGetBlockHash",
            FailedToDecode = "FailedToDecode",
            RollupAlreadyInitialized = "RollupAlreadyInitialized",
            RollupConfiguredByAnotherAccount = "RollupConfiguredByAnotherAccount"
        }

        export type Human = SubPriceFeed.Error$.Enum.BadOrigin
            | SubPriceFeed.Error$.Enum.NotConfigured
            | SubPriceFeed.Error$.Enum.InvalidKeyLength
            | SubPriceFeed.Error$.Enum.FailedToCreateClient
            | SubPriceFeed.Error$.Enum.FailedToCommitTx
            | SubPriceFeed.Error$.Enum.FailedToFetchPrice
            | SubPriceFeed.Error$.Enum.FailedToGetNameOwner
            | SubPriceFeed.Error$.Enum.FailedToClaimName
            | SubPriceFeed.Error$.Enum.FailedToGetStorage
            | SubPriceFeed.Error$.Enum.FailedToCreateTransaction
            | SubPriceFeed.Error$.Enum.FailedToSendTransaction
            | SubPriceFeed.Error$.Enum.FailedToGetBlockHash
            | SubPriceFeed.Error$.Enum.FailedToDecode
            | SubPriceFeed.Error$.Enum.RollupAlreadyInitialized
            | SubPriceFeed.Error$.Enum.RollupConfiguredByAnotherAccount;
        export type Codec = DPT.Enum<SubPriceFeed.Error$.Enum.BadOrigin, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.NotConfigured, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.InvalidKeyLength, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToCreateClient, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToCommitTx, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToFetchPrice, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToGetNameOwner, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToClaimName, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToGetStorage, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToCreateTransaction, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToSendTransaction, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToGetBlockHash, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.FailedToDecode, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.RollupAlreadyInitialized, never, never, PTT.Codec>
            | DPT.Enum<SubPriceFeed.Error$.Enum.RollupConfiguredByAnotherAccount, never, never, PTT.Codec>;
    }
}

export namespace PinkExtension {
    export namespace ChainExtension {
        export type PinkExt = any;

        export namespace PinkExt$ {
            export type Enum = any;
            export type Human = any;
            export type Codec = any;
        }
    }
}

export namespace SubPriceFeed {
    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Owner extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PTI.AccountId,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface Config extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                rpc: string | PT.Text,
                pallet_id: number | PT.U8,
                submit_key: number[] | string | PT.Vec<PT.U8>,
                token0: string | PT.Text,
                token1: string | PT.Text,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface TransferOwnership extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                new_owner: string | number[] | PTI.AccountId,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface MaybeInitRollup extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        SubPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface FeedPrice extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        SubPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }
    }

    interface MapMessageQuery extends DPT.MapMessageQuery {
        owner: ContractQuery.Owner;
        config: ContractQuery.Config;
        transferOwnership: ContractQuery.TransferOwnership;
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
            (options: ContractOptions, new_owner: string | number[]): DPT.SubmittableExtrinsic;
        }
    }

    interface MapMessageTx extends DPT.MapMessageTx {
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
    export declare class Factory extends DevPhase.ContractFactory<Contract> {
        instantiate(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<Contract>;
    }
}
