import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "@devphase/service";
import type * as DPT from "@devphase/service/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace LocalScheduler {
    type InkPrimitives_Types_AccountId$1 = any;
    type InkPrimitives_LangError$4 = {
        CouldNotReadInput? : null
        };
    type Result$2 = {
        Ok? : never[],
        Err? : InkPrimitives_LangError$4
        };
    type Result$5 = {
        Ok? : number,
        Err? : InkPrimitives_LangError$4
        };
    type LocalScheduler_LocalScheduler_JobConfig$8 = { name: string, cron_expr: string, target: InkPrimitives_Types_AccountId$1, call: number[] | string, enabled: boolean };
    type LocalScheduler_LocalScheduler_Error$9 = {
        BadOrigin? : null,
        JobNotFound? : null,
        NotChanged? : null,
        InvalidCronExpression? : null,
        CronExpressionNeverFire? : null,
        InternalErrorCacheCorrupted? : null,
        CallDataTooShort? : null,
        FailedToExecuteCall? : null,
        CalledJobReturnedError? : null
        };
    type Result$7 = {
        Ok? : LocalScheduler_LocalScheduler_JobConfig$8,
        Err? : LocalScheduler_LocalScheduler_Error$9
        };
    type Result$6 = {
        Ok? : Result$7,
        Err? : InkPrimitives_LangError$4
        };
    type Result$10 = {
        Ok? : number[] | string,
        Err? : InkPrimitives_LangError$4
        };
    type Result$12 = {
        Ok? : never[],
        Err? : LocalScheduler_LocalScheduler_Error$9
        };
    type Result$11 = {
        Ok? : Result$12,
        Err? : InkPrimitives_LangError$4
        };
    type Option$14 = {
        None? : null,
        Some? : [ number, LocalScheduler_LocalScheduler_JobConfig$8 ]
        };
    type Result$13 = {
        Ok? : Option$14,
        Err? : InkPrimitives_LangError$4
        };
    type Result$16 = {
        Ok? : InkPrimitives_Types_AccountId$1,
        Err? : InkPrimitives_LangError$4
        };
    type InkPrimitives_Types_Hash$17 = any;
    type PinkExtension_ChainExtension_PinkExt$18 = {

        };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetNumJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$5>>>;
        }

        export interface GetJob extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, idx: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$6>>>;
        }

        export interface GetActiveJobs extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$10>>>;
        }

        export interface GetJobSchedule extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, id: number): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$13>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$16>>>;
        }

        export interface Poll extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$11>>>;
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
            (options: ContractOptions, name: string, cron_expr: string, target: InkPrimitives_Types_AccountId$1, call: number[] | string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobCron extends DPT.ContractTx {
            (options: ContractOptions, id: number, cron_expr: string): DPT.SubmittableExtrinsic;
        }

        export interface SetJobTarget extends DPT.ContractTx {
            (options: ContractOptions, id: number, target: InkPrimitives_Types_AccountId$1, call: number[] | string): DPT.SubmittableExtrinsic;
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
