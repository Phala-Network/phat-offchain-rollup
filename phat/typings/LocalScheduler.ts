import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace LocalScheduler {
    type InkEnv_Types_AccountId = any;
    type InkPrimitives_Key = any;
    type InkStorage_Lazy_Mapping_Mapping = { offset_key: InkPrimitives_Key };
    type LocalScheduler_LocalScheduler_JobConfig = { name: string, cron_expr: string, target: InkEnv_Types_AccountId, call: number[], enabled: boolean };
    type LocalScheduler_LocalScheduler_Error = { BadOrigin: null } | { JobNotFound: null } | { NotChanged: null } | { InvalidCronExpression: null } | { CronExpressionNeverFire: null } | { InternalErrorCacheCorrupted: null } | { CallDataTooShort: null } | { FailedToCallJob: null };
    type Result = { Ok: never[] } | { Err: LocalScheduler_LocalScheduler_Error };
    type Option = { None: null } | { Some: [ number, LocalScheduler_LocalScheduler_JobConfig ] };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetNumJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.INumber>>;
        }

        export interface GetJob extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, idx: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result>>>;
        }

        export interface GetActiveJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IVec<DPT.INumber>>>;
        }

        export interface GetJobSchedule extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, id: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Option>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<InkEnv_Types_AccountId>>>;
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
            (options: ContractOptions, name: string, cron_expr: string, target: InkEnv_Types_AccountId, call: number[]): DPT.SubmittableExtrinsic;
        }

        export interface SetJobCron extends DPT.ContractTx {
            (options: ContractOptions, id: number, cron_expr: string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobTarget extends DPT.ContractTx {
            (options: ContractOptions, id: number, target: InkEnv_Types_AccountId, call: number[]): DPT.SubmittableExtrinsic;
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
