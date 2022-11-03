import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "devphase";
import type * as DPT from "devphase/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace Sample_oracle {
    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface RollupHandler$$handle_rollup extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<any | DPT.IVec<DPT.INumber>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        RollupHandler$$handle_rollup: ContractQuery.RollupHandler$$handle_rollup;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, anchor: { undefined: DPT.FixedArray<number, 20> }): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
        config: ContractTx.Config;
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
