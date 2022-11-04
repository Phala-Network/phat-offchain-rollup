import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace EvmTransactor {
    type ink_env$types$AccountId = any;
    type primitive_types$H160 = any;

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<ink_env$types$AccountId>>;
        }

        export interface Wallet extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<primitive_types$H160>>;
        }

        export interface Get_retired_secret_key extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<any>>;
        }

        export interface Poll extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<any>>;
        }

        export interface Test_poll_with_key extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, key: DPT.FixedArray<number, 32>): DPT.CallResult<DPT.CallOutcome<any>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        owner: ContractQuery.Owner;
        wallet: ContractQuery.Wallet;
        get_retired_secret_key: ContractQuery.Get_retired_secret_key;
        poll: ContractQuery.Poll;
        test_poll_with_key: ContractQuery.Test_poll_with_key;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, rollup_handler: ink_env$types$AccountId, anchor: primitive_types$H160): DPT.SubmittableExtrinsic;
        }

        export interface Retire_wallet extends DPT.ContractTx {
            (options: ContractOptions): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
        config: ContractTx.Config;
        retire_wallet: ContractTx.Retire_wallet;
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
