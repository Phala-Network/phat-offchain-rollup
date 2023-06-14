import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "@devphase/service";
import type * as DPT from "@devphase/service/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace Sub0Factory {
    type InkPrimitives_Types_AccountId$1 = any;
    type InkPrimitives_Types_Hash$2 = any;
    type InkPrimitives_LangError$5 = {
        CouldNotReadInput? : null
        };
    type Result$3 = {
        Ok? : never[],
        Err? : InkPrimitives_LangError$5
        };
    type Sub0Factory_Sub0Factory_Error$8 = {
        BadOrigin? : null,
        NotConfigured? : null,
        InvalidKeyLength? : null,
        FailedToDeployContract? : null,
        FailedToConfigContract? : null,
        FailedToTransferOwnership? : null
        };
    type Result$7 = {
        Ok? : never[],
        Err? : Sub0Factory_Sub0Factory_Error$8
        };
    type Result$6 = {
        Ok? : Result$7,
        Err? : InkPrimitives_LangError$5
        };
    type Result$10 = {
        Ok? : [ number, InkPrimitives_Types_Hash$2 ],
        Err? : Sub0Factory_Sub0Factory_Error$8
        };
    type Result$9 = {
        Ok? : Result$10,
        Err? : InkPrimitives_LangError$5
        };
    type Result$13 = {
        Ok? : InkPrimitives_Types_AccountId$1,
        Err? : Sub0Factory_Sub0Factory_Error$8
        };
    type Result$12 = {
        Ok? : Result$13,
        Err? : InkPrimitives_LangError$5
        };
    type Sub0Factory_Sub0Factory_Deployment$16 = { name: string, owner: InkPrimitives_Types_AccountId$1, contract_id: InkPrimitives_Types_AccountId$1, created_at: number, expired_at: number };
    type Result$15 = {
        Ok? : Sub0Factory_Sub0Factory_Deployment$16[],
        Err? : Sub0Factory_Sub0Factory_Error$8
        };
    type Result$14 = {
        Ok? : Result$15,
        Err? : InkPrimitives_LangError$5
        };
    type Result$17 = {
        Ok? : InkPrimitives_Types_AccountId$1,
        Err? : InkPrimitives_LangError$5
        };
    type PinkExtension_ChainExtension_PinkExt$18 = {

        };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface GetConfig extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$9>>>;
        }

        export interface GetDeployments extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$14>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$17>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        getConfig: ContractQuery.GetConfig;
        getDeployments: ContractQuery.GetDeployments;
        owner: ContractQuery.Owner;
    }

    /** */
    /** Transactions */
    /** */
    namespace ContractTx {
        export interface Config extends DPT.ContractTx {
            (options: ContractOptions, rpc: string, pallet_id: number, submit_key: number[] | string, price_feed_code: InkPrimitives_Types_Hash$2): DPT.SubmittableExtrinsic;
        }

        export interface DeployPriceFeed extends DPT.ContractTx {
            (options: ContractOptions, name: string, token0: string, token1: string): DPT.SubmittableExtrinsic;
        }
    }

    export interface MapMessageTx extends DPT.MapMessageTx {
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
    export declare class Factory extends DevPhase.ContractFactory {
        instantiate<T = Contract>(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<T>;
    }
}
