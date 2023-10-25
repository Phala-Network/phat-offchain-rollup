import type * as PhalaSdk from "@phala/sdk";
import type * as DevPhase from "@devphase/service";
import type * as DPT from "@devphase/service/etc/typings";
import type { ContractCallResult, ContractQuery } from "@polkadot/api-contract/base/types";
import type { ContractCallOutcome, ContractOptions } from "@polkadot/api-contract/types";
import type { Codec } from "@polkadot/types/types";

export namespace MomokaPublication {
    type InkPrimitives_Types_AccountId$1 = any;
    type InkPrimitives_LangError$4 = {
        CouldNotReadInput? : null
        };
    type Result$2 = {
        Ok? : never[],
        Err? : InkPrimitives_LangError$4
        };
    type Result$5 = {
        Ok? : [ number, number, number ],
        Err? : InkPrimitives_LangError$4
        };
    type Result$7 = {
        Ok? : InkPrimitives_Types_AccountId$1,
        Err? : InkPrimitives_LangError$4
        };
    type PrimitiveTypes_H160$9 = any;
    type Result$8 = {
        Ok? : PrimitiveTypes_H160$9,
        Err? : InkPrimitives_LangError$4
        };
    type MomokaPublication_MomokaPublication_Client$12 = { rpc: string, client_addr: DPT.FixedArray<number, 20> };
    type MomokaPublication_MomokaPublication_Error$13 = {
        BadOrigin? : null,
        ClientNotConfigured? : null,
        InvalidClientAddress? : null,
        FailedToFetchData? : null,
        FailedToDecode? : null,
        FailedToParseJson? : null,
        PublicationNotExists? : null,
        FailedToParseId? : null,
        FailedToParseAddress? : null,
        NoProofForComment? : null,
        UnknownPublicationType? : null,
        MissingMirrorField? : null,
        MissingCollectModule? : null,
        BadEvmAnchorAbi? : null,
        EvmFailedToPrepareMetaTx? : null,
        BadPublicationId? : null,
        BadDaId? : null
        };
    type Result$11 = {
        Ok? : MomokaPublication_MomokaPublication_Client$12,
        Err? : MomokaPublication_MomokaPublication_Error$13
        };
    type Result$10 = {
        Ok? : Result$11,
        Err? : InkPrimitives_LangError$4
        };
    type Result$15 = {
        Ok? : never[],
        Err? : MomokaPublication_MomokaPublication_Error$13
        };
    type Result$14 = {
        Ok? : Result$15,
        Err? : InkPrimitives_LangError$4
        };
    type Result$17 = {
        Ok? : number[] | string,
        Err? : MomokaPublication_MomokaPublication_Error$13
        };
    type Result$16 = {
        Ok? : Result$17,
        Err? : InkPrimitives_LangError$4
        };
    type InkPrimitives_Types_Hash$18 = any;
    type PinkExtension_ChainExtension_PinkExt$19 = {

        };

    /** */
    /** Queries */
    /** */
    namespace ContractQuery {
        export interface Version extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$5>>>;
        }

        export interface Owner extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$7>>>;
        }

        export interface GetAttestAddress extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$8>>>;
        }

        export interface GetClient extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$10>>>;
        }

        export interface CheckLensPublication extends DPT.ContractQuery {
            (certificateData: PhalaSdk.CertificateData, options: ContractOptions, publication_id: string, mainnet: boolean): DPT.CallResult<DPT.CallOutcome<DPT.IJson<Result$16>>>;
        }
    }

    export interface MapMessageQuery extends DPT.MapMessageQuery {
        version: ContractQuery.Version;
        owner: ContractQuery.Owner;
        getAttestAddress: ContractQuery.GetAttestAddress;
        getClient: ContractQuery.GetClient;
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

    export interface MapMessageTx extends DPT.MapMessageTx {
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
    export declare class Factory extends DevPhase.ContractFactory {
        instantiate<T = Contract>(constructor: "default", params: never[], options?: DevPhase.InstantiateOptions): Promise<T>;
        instantiate<T = Contract>(constructor: "new", params: [DPT.FixedArray<number, 32>], options?: DevPhase.InstantiateOptions): Promise<T>;
    }
}
