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

export namespace Sub0Factory {
    export interface Error {
        badOrigin?: null;
        notConfigured?: null;
        invalidKeyLength?: null;
        failedToDeployContract?: null;
        failedToConfigContract?: null;
        failedToTransferOwnership?: null;
        [index: string]: any;
    }

    export interface Deployment {
        name: string;
        owner: string | number[];
        contractId: string | number[];
        createdAt: number;
        expiredAt: number;
    }

    export namespace Error$ {
        export enum Enum {
            BadOrigin = "BadOrigin",
            NotConfigured = "NotConfigured",
            InvalidKeyLength = "InvalidKeyLength",
            FailedToDeployContract = "FailedToDeployContract",
            FailedToConfigContract = "FailedToConfigContract",
            FailedToTransferOwnership = "FailedToTransferOwnership"
        }

        export type Human = Sub0Factory.Error$.Enum.BadOrigin & { [index: string]: any }
            | Sub0Factory.Error$.Enum.NotConfigured & { [index: string]: any }
            | Sub0Factory.Error$.Enum.InvalidKeyLength & { [index: string]: any }
            | Sub0Factory.Error$.Enum.FailedToDeployContract & { [index: string]: any }
            | Sub0Factory.Error$.Enum.FailedToConfigContract & { [index: string]: any }
            | Sub0Factory.Error$.Enum.FailedToTransferOwnership & { [index: string]: any };

        export interface Codec extends PT.Enum {
            type: Enum;
            inner: PTT.Codec;
            value: PTT.Codec;
            toHuman(isExtended?: boolean): Human;
            toJSON(): Error;
            toPrimitive(): Error;
        }
    }

    export namespace Deployment$ {
        export interface Human {
            name: string;
            owner: string;
            contractId: string;
            createdAt: number;
            expiredAt: number;
        }

        export interface Codec extends DPT.Json<Sub0Factory.Deployment, Sub0Factory.Deployment$.Human> {
            name: PT.Text;
            owner: PTI.AccountId;
            contractId: PTI.AccountId;
            createdAt: PT.U64;
            expiredAt: PT.U64;
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

export namespace Sub0Factory {
    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Config extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                rpc: string | PT.Text,
                pallet_id: number | PT.U8,
                submit_key: number[] | string | PT.Vec<PT.U8>,
                price_feed_code: string | number[] | PTI.Hash,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface GetConfig extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        PTT.ITuple<[PT.U8, PTI.Hash]>,
                        Sub0Factory.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface DeployPriceFeed extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                name: string | PT.Text,
                token0: string | PT.Text,
                token1: string | PT.Text,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface GetDeployments extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        PT.Vec<Sub0Factory.Deployment$.Codec>,
                        Sub0Factory.Error$.Codec
                    >,
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
    }

    interface MapMessageQuery extends DPT.MapMessageQuery {
        config: ContractQuery.Config;
        getConfig: ContractQuery.GetConfig;
        deployPriceFeed: ContractQuery.DeployPriceFeed;
        getDeployments: ContractQuery.GetDeployments;
        owner: ContractQuery.Owner;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, pallet_id: number, submit_key: number[] | string, price_feed_code: string | number[]): DPT.SubmittableExtrinsic;
        }

        export interface DeployPriceFeed extends DPT.ContractTx {
            (options: ContractOptions, name: string, token0: string, token1: string): DPT.SubmittableExtrinsic;
        }
    }

    interface MapMessageTx extends DPT.MapMessageTx {
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
    export declare class Factory extends DevPhase.ContractFactory<Contract> {
        instantiate(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<Contract>;
    }
}
