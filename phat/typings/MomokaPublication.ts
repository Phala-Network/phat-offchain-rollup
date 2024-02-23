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

export namespace MomokaPublication {
    export interface Client {
        rpc: string;
        clientAddr: number[];
    }

    export interface Error {
        badOrigin?: null;
        clientNotConfigured?: null;
        invalidClientAddress?: null;
        failedToFetchData?: null;
        failedToDecode?: null;
        failedToParseJson?: null;
        publicationNotExists?: null;
        failedToParseId?: null;
        failedToParseAddress?: null;
        noProofForComment?: null;
        noProofForQuote?: null;
        unknownPublicationType?: null;
        missingMirrorField?: null;
        missingCollectModule?: null;
        badEvmAnchorAbi?: null;
        evmFailedToPrepareMetaTx?: null;
        badPublicationId?: null;
        badDaId?: null;
        [index: string]: any;
    }

    export namespace Client$ {
        export interface Human {
            rpc: string;
            clientAddr: number[];
        }

        export interface Codec extends DPT.Json<MomokaPublication.Client, MomokaPublication.Client$.Human> {
            rpc: PT.Text;
            clientAddr: PT.VecFixed<PT.U8>;
        }
    }

    export namespace Error$ {
        export enum Enum {
            BadOrigin = "BadOrigin",
            ClientNotConfigured = "ClientNotConfigured",
            InvalidClientAddress = "InvalidClientAddress",
            FailedToFetchData = "FailedToFetchData",
            FailedToDecode = "FailedToDecode",
            FailedToParseJson = "FailedToParseJson",
            PublicationNotExists = "PublicationNotExists",
            FailedToParseId = "FailedToParseId",
            FailedToParseAddress = "FailedToParseAddress",
            NoProofForComment = "NoProofForComment",
            NoProofForQuote = "NoProofForQuote",
            UnknownPublicationType = "UnknownPublicationType",
            MissingMirrorField = "MissingMirrorField",
            MissingCollectModule = "MissingCollectModule",
            BadEvmAnchorAbi = "BadEvmAnchorAbi",
            EvmFailedToPrepareMetaTx = "EvmFailedToPrepareMetaTx",
            BadPublicationId = "BadPublicationId",
            BadDaId = "BadDaId"
        }

        export type Human = MomokaPublication.Error$.Enum.BadOrigin & { [index: string]: any }
            | MomokaPublication.Error$.Enum.ClientNotConfigured & { [index: string]: any }
            | MomokaPublication.Error$.Enum.InvalidClientAddress & { [index: string]: any }
            | MomokaPublication.Error$.Enum.FailedToFetchData & { [index: string]: any }
            | MomokaPublication.Error$.Enum.FailedToDecode & { [index: string]: any }
            | MomokaPublication.Error$.Enum.FailedToParseJson & { [index: string]: any }
            | MomokaPublication.Error$.Enum.PublicationNotExists & { [index: string]: any }
            | MomokaPublication.Error$.Enum.FailedToParseId & { [index: string]: any }
            | MomokaPublication.Error$.Enum.FailedToParseAddress & { [index: string]: any }
            | MomokaPublication.Error$.Enum.NoProofForComment & { [index: string]: any }
            | MomokaPublication.Error$.Enum.NoProofForQuote & { [index: string]: any }
            | MomokaPublication.Error$.Enum.UnknownPublicationType & { [index: string]: any }
            | MomokaPublication.Error$.Enum.MissingMirrorField & { [index: string]: any }
            | MomokaPublication.Error$.Enum.MissingCollectModule & { [index: string]: any }
            | MomokaPublication.Error$.Enum.BadEvmAnchorAbi & { [index: string]: any }
            | MomokaPublication.Error$.Enum.EvmFailedToPrepareMetaTx & { [index: string]: any }
            | MomokaPublication.Error$.Enum.BadPublicationId & { [index: string]: any }
            | MomokaPublication.Error$.Enum.BadDaId & { [index: string]: any };

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

export namespace MomokaPublication {
    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Version extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PTT.ITuple<[PT.U16, PT.U16, PT.U16]>,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

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
                    PTI.H160,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface GetClient extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        MomokaPublication.Client$.Codec,
                        MomokaPublication.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface ConfigClient extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                rpc: string | PT.Text,
                client_addr: number[] | string | PT.Vec<PT.U8>,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface CheckLensPublication extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                publication_id: string | PT.Text,
                mainnet: boolean | PT.Bool,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        PT.Vec<PT.U8>,
                        MomokaPublication.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }
    }

    interface MapMessageQuery extends DPT.MapMessageQuery {
        version: ContractQuery.Version;
        owner: ContractQuery.Owner;
        getAttestAddress: ContractQuery.GetAttestAddress;
        getClient: ContractQuery.GetClient;
        configClient: ContractQuery.ConfigClient;
        checkLensPublication: ContractQuery.CheckLensPublication;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface ConfigClient extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, client_addr: number[] | string): DPT.SubmittableExtrinsic;
        }
    }

    interface MapMessageTx extends DPT.MapMessageTx {
        configClient: ContractTx.ConfigClient;
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
        instantiate(constructor: "new", params: [number[] | PT.VecFixed<PT.U8>], options?: DevPhase.InstantiateOptions): Promise<Contract>;
    }
}
