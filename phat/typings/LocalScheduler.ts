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

export namespace LocalScheduler {
    export interface JobConfig {
        name: string;
        cronExpr: string;
        target: string | number[];
        call: number[] | string;
        enabled: boolean;
    }

    export interface Error {
        badOrigin?: null;
        jobNotFound?: null;
        notChanged?: null;
        invalidCronExpression?: null;
        cronExpressionNeverFire?: null;
        internalErrorCacheCorrupted?: null;
        callDataTooShort?: null;
        failedToExecuteCall?: null;
        calledJobReturnedError?: null;
        [index: string]: any;
    }

    export namespace JobConfig$ {
        export interface Human {
            name: string;
            cronExpr: string;
            target: string;
            call: number[] | string;
            enabled: boolean;
        }

        export interface Codec extends DPT.Json<LocalScheduler.JobConfig, LocalScheduler.JobConfig$.Human> {
            name: PT.Text;
            cronExpr: PT.Text;
            target: PTI.AccountId;
            call: PT.Vec<PT.U8>;
            enabled: PT.Bool;
        }
    }

    export namespace Error$ {
        export enum Enum {
            BadOrigin = "BadOrigin",
            JobNotFound = "JobNotFound",
            NotChanged = "NotChanged",
            InvalidCronExpression = "InvalidCronExpression",
            CronExpressionNeverFire = "CronExpressionNeverFire",
            InternalErrorCacheCorrupted = "InternalErrorCacheCorrupted",
            CallDataTooShort = "CallDataTooShort",
            FailedToExecuteCall = "FailedToExecuteCall",
            CalledJobReturnedError = "CalledJobReturnedError"
        }

        export type Human = LocalScheduler.Error$.Enum.BadOrigin & { [index: string]: any }
            | LocalScheduler.Error$.Enum.JobNotFound & { [index: string]: any }
            | LocalScheduler.Error$.Enum.NotChanged & { [index: string]: any }
            | LocalScheduler.Error$.Enum.InvalidCronExpression & { [index: string]: any }
            | LocalScheduler.Error$.Enum.CronExpressionNeverFire & { [index: string]: any }
            | LocalScheduler.Error$.Enum.InternalErrorCacheCorrupted & { [index: string]: any }
            | LocalScheduler.Error$.Enum.CallDataTooShort & { [index: string]: any }
            | LocalScheduler.Error$.Enum.FailedToExecuteCall & { [index: string]: any }
            | LocalScheduler.Error$.Enum.CalledJobReturnedError & { [index: string]: any };

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

export namespace LocalScheduler {
    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetNumJobs extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PT.U32,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface GetJob extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                idx: number | PT.U32,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        LocalScheduler.JobConfig$.Codec,
                        LocalScheduler.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface GetActiveJobs extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    PT.Vec<PT.U32>,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }

        export interface AddJob extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                name: string | PT.Text,
                cron_expr: string | PT.Text,
                target: string | number[] | PTI.AccountId,
                call: number[] | string | PT.Vec<PT.U8>,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface SetJobCron extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                id: number | PT.U32,
                cron_expr: string | PT.Text,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface SetJobTarget extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                id: number | PT.U32,
                target: string | number[] | PTI.AccountId,
                call: number[] | string | PT.Vec<PT.U8>,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface SetJobEnabled extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                id: number | PT.U32,
                enabled: boolean | PT.Bool,
            ): DPT.CallReturn<
                ContractExecResult
            >;
        }

        export interface GetJobSchedule extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
                id: number | PT.U32,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Option$.Codec<
                        PTT.ITuple<[PT.U64, LocalScheduler.JobConfig$.Codec]>
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

        export interface Poll extends DPT.ContractQuery {
            (
                origin: DPT.ContractCallOrigin,
                options: DPT.ContractCallOptions,
            ): DPT.CallReturn<
                DPT.Result$.Codec<
                    DPT.Result$.Codec<
                        PTT.ITuple<[]>,
                        LocalScheduler.Error$.Codec
                    >,
                    InkPrimitives.LangError$.Codec
                >
            >;
        }
    }

    interface MapMessageQuery extends DPT.MapMessageQuery {
        getNumJobs: ContractQuery.GetNumJobs;
        getJob: ContractQuery.GetJob;
        getActiveJobs: ContractQuery.GetActiveJobs;
        addJob: ContractQuery.AddJob;
        setJobCron: ContractQuery.SetJobCron;
        setJobTarget: ContractQuery.SetJobTarget;
        setJobEnabled: ContractQuery.SetJobEnabled;
        getJobSchedule: ContractQuery.GetJobSchedule;
        owner: ContractQuery.Owner;
        poll: ContractQuery.Poll;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface AddJob extends DPT.ContractTx {
            (options: ContractOptions, name: string, cron_expr: string, target: string | number[], call: number[] | string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobCron extends DPT.ContractTx {
            (options: ContractOptions, id: number, cron_expr: string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobTarget extends DPT.ContractTx {
            (options: ContractOptions, id: number, target: string | number[], call: number[] | string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobEnabled extends DPT.ContractTx {
            (options: ContractOptions, id: number, enabled: boolean): DPT.SubmittableExtrinsic;
        }
    }

    interface MapMessageTx extends DPT.MapMessageTx {
        addJob: ContractTx.AddJob;
        setJobCron: ContractTx.SetJobCron;
        setJobTarget: ContractTx.SetJobTarget;
        setJobEnabled: ContractTx.SetJobEnabled;
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
