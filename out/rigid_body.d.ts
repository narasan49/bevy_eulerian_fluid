/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h04da1a941b61fa75: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h069515cf1da1aef3: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__haa042f17fba0a09d: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h9fa514c4a33507b4: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h2402c4d1f5effacf: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hb6093a228a526adf: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h2c53c8b113884d0b: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h539eb28742531e51: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__ha40dc2305e13d987: (a: number, b: number) => void;
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
