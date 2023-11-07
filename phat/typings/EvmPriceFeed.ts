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

export namespace EvmPriceFeed {
    export interface Error {
        badOrigin?: null;
        notConfigured?: null;
        invalidKeyLength?: null;
        invalidAddressLength?: null;
        noRequestInQueue?: null;
        failedToCreateClient?: null;
        failedToCommitTx?: null;
        failedToFetchPrice?: null;
        failedToGetStorage?: null;
        failedToCreateTransaction?: null;
        failedToSendTransaction?: null;
        failedToGetBlockHash?: null;
        failedToDecode?: null;
        invalidRequest?: null;
    }

    export namespace Error$ {
        export enum Enum {
            BadOrigin = "BadOrigin",
            NotConfigured = "NotConfigured",
            InvalidKeyLength = "InvalidKeyLength",
            InvalidAddressLength = "InvalidAddressLength",
            NoRequestInQueue = "NoRequestInQueue",
            FailedToCreateClient = "FailedToCreateClient",
            FailedToCommitTx = "FailedToCommitTx",
            FailedToFetchPrice = "FailedToFetchPrice",
            FailedToGetStorage = "FailedToGetStorage",
            FailedToCreateTransaction = "FailedToCreateTransaction",
            FailedToSendTransaction = "FailedToSendTransaction",
            FailedToGetBlockHash = "FailedToGetBlockHash",
            FailedToDecode = "FailedToDecode",
            InvalidRequest = "InvalidRequest"
        }

        export type Human = EvmPriceFeed.Error$.Enum.BadOrigin
            | EvmPriceFeed.Error$.Enum.NotConfigured
            | EvmPriceFeed.Error$.Enum.InvalidKeyLength
            | EvmPriceFeed.Error$.Enum.InvalidAddressLength
            | EvmPriceFeed.Error$.Enum.NoRequestInQueue
            | EvmPriceFeed.Error$.Enum.FailedToCreateClient
            | EvmPriceFeed.Error$.Enum.FailedToCommitTx
            | EvmPriceFeed.Error$.Enum.FailedToFetchPrice
            | EvmPriceFeed.Error$.Enum.FailedToGetStorage
            | EvmPriceFeed.Error$.Enum.FailedToCreateTransaction
            | EvmPriceFeed.Error$.Enum.FailedToSendTransaction
            | EvmPriceFeed.Error$.Enum.FailedToGetBlockHash
            | EvmPriceFeed.Error$.Enum.FailedToDecode
            | EvmPriceFeed.Error$.Enum.InvalidRequest;
        export type Codec = DPT.Enum<EvmPriceFeed.Error$.Enum.BadOrigin, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.NotConfigured, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.InvalidKeyLength, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.InvalidAddressLength, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.NoRequestInQueue, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToCreateClient, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToCommitTx, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToFetchPrice, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToGetStorage, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToCreateTransaction, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToSendTransaction, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToGetBlockHash, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.FailedToDecode, never, never, PTT.Codec>
            | DPT.Enum<EvmPriceFeed.Error$.Enum.InvalidRequest, never, never, PTT.Codec>;
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

export namespace EvmPriceFeed {
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
                anchor_addr: number[] | string | PT.Vec<PT.U8>,
                attest_key: number[] | string | PT.Vec<PT.U8>,
                sender_key: DPT.Option<
                    number[] | string
                > | DPT.Option$.Codec<
                    PT.Vec<PT.U8>
                >,
                token0: string | PT.Text,
                token1: string | PT.Text,
                feed_id: number | PT.U32,
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
                        EvmPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface AnswerPrice extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        EvmPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface FeedCustomPrice extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                rpc: string | PT.Text,
                anchor_addr: number[] | PT.VecFixed<PT.U8>,
                attest_key: number[] | PT.VecFixed<PT.U8>,
                sender_key: DPT.Option<
                    number[]
                > | DPT.Option$.Codec<
                    PT.VecFixed<PT.U8>
                >,
                feed_id: number | PT.U32,
                price: number | PT.U128,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        EvmPriceFeed.Error$.Codec
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
        feedPrice: ContractQuery.FeedPrice;
        answerPrice: ContractQuery.AnswerPrice;
        feedCustomPrice: ContractQuery.FeedCustomPrice;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, anchor_addr: number[] | string, attest_key: number[] | string, sender_key: DPT.Option<
                number[] | string
            >, token0: string, token1: string, feed_id: number): DPT.SubmittableExtrinsic;
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
