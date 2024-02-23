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

export namespace InkPriceFeed {
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
        failedToCallRollup?: null;
        [index: string]: any;
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
            InvalidRequest = "InvalidRequest",
            FailedToCallRollup = "FailedToCallRollup"
        }

        export type Human = InkPriceFeed.Error$.Enum.BadOrigin & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.NotConfigured & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.InvalidKeyLength & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.InvalidAddressLength & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.NoRequestInQueue & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToCreateClient & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToCommitTx & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToFetchPrice & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToGetStorage & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToCreateTransaction & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToSendTransaction & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToGetBlockHash & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToDecode & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.InvalidRequest & { [index: string]: any }
            | InkPriceFeed.Error$.Enum.FailedToCallRollup & { [index: string]: any };

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

export namespace InkPriceFeed {
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

        export interface GetAttestAddress extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PT.Vec<PT.U8>,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface GetAttestAddressMetaTx extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PT.Vec<PT.U8>,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface SetAttestKey extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                attest_key: DPT.Option<
                    number[] | string
                > | DPT.Option$.Codec<
                    PT.Vec<PT.U8>
                >,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface GetSenderAddress extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Option$.Codec<
                        PT.Vec<PT.U8>
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface GetTargetContract extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Option$.Codec<
                        PTT.ITuple<[PT.Text, PT.U8, PT.U8, PT.VecFixed<PT.U8>]>
                    >,
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
                call_id: number | PT.U8,
                contract_id: number[] | string | PT.Vec<PT.U8>,
                sender_key: DPT.Option<
                    number[] | string
                > | DPT.Option$.Codec<
                    PT.Vec<PT.U8>
                >,
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

        export interface FeedPriceFromCoingecko extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                trading_pair_id: number | PT.U32,
                token0: string | PT.Text,
                token1: string | PT.Text,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        InkPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface FeedCustomPrice extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                trading_pair_id: number | PT.U32,
                price: number | PT.U128,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        InkPriceFeed.Error$.Codec
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
                        InkPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface AnswerPriceWithConfig extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                rpc: string | PT.Text,
                pallet_id: number | PT.U8,
                call_id: number | PT.U8,
                contract_id: number[] | string | PT.Vec<PT.U8>,
                sender_key: DPT.Option<
                    number[] | string
                > | DPT.Option$.Codec<
                    PT.Vec<PT.U8>
                >,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        DPT.Option$.Codec<
                            PT.Vec<PT.U8>
                        >,
                        InkPriceFeed.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }
    }

    interface MapMessageQuery extends DPT.MapMessageQuery {
        owner: ContractQuery.Owner;
        getAttestAddress: ContractQuery.GetAttestAddress;
        getAttestAddressMetaTx: ContractQuery.GetAttestAddressMetaTx;
        setAttestKey: ContractQuery.SetAttestKey;
        getSenderAddress: ContractQuery.GetSenderAddress;
        getTargetContract: ContractQuery.GetTargetContract;
        config: ContractQuery.Config;
        transferOwnership: ContractQuery.TransferOwnership;
        feedPriceFromCoingecko: ContractQuery.FeedPriceFromCoingecko;
        feedCustomPrice: ContractQuery.FeedCustomPrice;
        answerPrice: ContractQuery.AnswerPrice;
        answerPriceWithConfig: ContractQuery.AnswerPriceWithConfig;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface SetAttestKey extends DPT.ContractTx {
            (options: ContractOptions, attest_key: DPT.Option<
                number[] | string
            >): DPT.SubmittableExtrinsic;
        }

        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, pallet_id: number, call_id: number, contract_id: number[] | string, sender_key: DPT.Option<
                number[] | string
            >): DPT.SubmittableExtrinsic;
        }

        export interface TransferOwnership extends DPT.ContractTx {
            (options: ContractOptions, new_owner: string | number[]): DPT.SubmittableExtrinsic;
        }
    }

    interface MapMessageTx extends DPT.MapMessageTx {
        setAttestKey: ContractTx.SetAttestKey;
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
