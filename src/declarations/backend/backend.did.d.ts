import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Classification { 'label' : string, 'score' : number }
export interface ClassificationError { 'message' : string }
export type ClassificationResult = { 'Ok' : Array<Classification> } |
  { 'Err' : ClassificationError };
export type HeaderField = [string, string];
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<HeaderField>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<HeaderField>,
  'status_code' : number,
}
export interface _SERVICE {
  'classify' : ActorMethod<[Uint8Array | number[]], ClassificationResult>,
  'classify_query' : ActorMethod<[Uint8Array | number[]], ClassificationResult>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'llm' : ActorMethod<[string], string>,
  'run' : ActorMethod<[], ClassificationResult>,
  'send_http_post_request' : ActorMethod<[string], string>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
