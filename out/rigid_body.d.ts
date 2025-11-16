/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h7399bc1d08e242ab: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hebdefb0b7e1e5ed1: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h0a4f1a70d7b36c9f: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h122c1cb7638a2c55: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h8022a82b1100aad0: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h1f85db3e71ec8e18: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h9d41a339209a6487: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h07a084f03a7f1591: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hc32aa7836dfa26be: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
