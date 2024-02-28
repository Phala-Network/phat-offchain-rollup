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
        [index: string]: any;
    }

    export namespace LangError$ {
        export enum Enum {
            CouldNotReadInput = "CouldNotReadInput"
        }

        export type Human = InkPrimitives.LangError$.Enum.CouldNotReadInput & { [index: string]: any };

        export interface Codec extends PT.Enum {
            type: Enum;
            inner: PTT.Codec;
            value: PTT.Codec;
            toHuman(isExtended?: boolean): Human;
            toJSON(): LangError;
            toPrimitive(): LangError;
        }
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
        [index: string]: any;
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

        export type Human = SubPriceFeed.Error$.Enum.BadOrigin & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.NotConfigured & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.InvalidKeyLength & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToCreateClient & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToCommitTx & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToFetchPrice & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToGetNameOwner & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToClaimName & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToGetStorage & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToCreateTransaction & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToSendTransaction & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToGetBlockHash & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.FailedToDecode & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.RollupAlreadyInitialized & { [index: string]: any }
            | SubPriceFeed.Error$.Enum.RollupConfiguredByAnotherAccount & { [index: string]: any };

        export interface Codec extends PT.Enum {
            type: Enum;
            inner: PTT.Codec;
            value: PTT.Codec;
            toHuman(isExtended?: boolean): Human;
            toJSON(): Error;
            toPrimitive(): Error;
        }
    }
}

export namespace Pink {
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
