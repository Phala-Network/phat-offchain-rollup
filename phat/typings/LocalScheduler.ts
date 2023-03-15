import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace LocalScheduler {
    type InkPrimitives_Types_AccountId = any;
    type InkPrimitives_LangError = { CouldNotReadInput: null };
    type Result = { Ok: InkPrimitives_Types_AccountId } | { Err: InkPrimitives_LangError };
    type LocalScheduler_LocalScheduler_JobConfig = { name: string, cron_expr: string, target: InkPrimitives_Types_AccountId, call: number[], enabled: boolean };
    type LocalScheduler_LocalScheduler_Error = { BadOrigin: null } | { JobNotFound: null } | { NotChanged: null } | { InvalidCronExpression: null } | { CronExpressionNeverFire: null } | { InternalErrorCacheCorrupted: null } | { CallDataTooShort: null } | { FailedToCallJob: null };
    type Option = { None: null } | { Some: [ number, LocalScheduler_LocalScheduler_JobConfig ] };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetNumJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface GetJob extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, idx: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface GetActiveJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface GetJobSchedule extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, id: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface Poll extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        getNumJobs: ContractQuery.GetNumJobs;
        getJob: ContractQuery.GetJob;
        getActiveJobs: ContractQuery.GetActiveJobs;
        getJobSchedule: ContractQuery.GetJobSchedule;
        owner: ContractQuery.Owner;
        poll: ContractQuery.Poll;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface AddJob extends DPT.ContractTx {
            (options: ContractOptions, name: string, cron_expr: string, target: InkPrimitives_Types_AccountId, call: number[]): DPT.SubmittableExtrinsic;
        }

        export interface SetJobCron extends DPT.ContractTx {
            (options: ContractOptions, id: number, cron_expr: string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobTarget extends DPT.ContractTx {
            (options: ContractOptions, id: number, target: InkPrimitives_Types_AccountId, call: number[]): DPT.SubmittableExtrinsic;
        }

        export interface SetJobEnabled extends DPT.ContractTx {
            (options: ContractOptions, id: number, enabled: boolean): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
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
    export declare class Factory extends DevPhase.ContractFactory {
        instantiate<T = Contract>(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<T>;
    }
}
