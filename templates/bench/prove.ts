/*
 * mCaptcha is a PoW based DoS protection software.
 * This is the frontend web component of the mCaptcha system
 * Copyright © 2021 Aravinth Manivnanan <realaravinth@batsense.net>.
 *
 * Use of this source code is governed by Apache 2.0 or MIT license.
 * You shoud have received a copy of MIT and Apache 2.0 along with
 * this program. If not, see <https://spdx.org/licenses/MIT.html> for
 * MIT or <http://www.apache.org/licenses/LICENSE-2.0> for Apache.
 */

import * as p from "@mcaptcha/pow_sha256-polyfill";
import { PoWConfig, SubmissionType } from "./types";

export const get_bench_type = async (): Promise<SubmissionType> => {
  console.log(`Wasm support says ${WasmSupported}`);
  let submission_type: SubmissionType;
  if (WasmSupported) {
    submission_type = SubmissionType.wasm;
  } else {
    submission_type = SubmissionType.js;
  }
  return submission_type;
};

/**
 * proove work
 * @param {PoWConfig} config - the proof-of-work configuration using which
 * work needs to be computed
 * */
const prove = async (config: PoWConfig): Promise<number> => {
  console.log(`Wasm support says ${WasmSupported}`);
  let duration: number;
  if (WasmSupported) {
    const wasm = await require("@mcaptcha/pow-wasm");
    const t0 = performance.now();
    console.log(JSON.stringify(wasm))
    wasm.gen_pow(config.salt, config.string, config.difficulty_factor);
    const t1 = performance.now();
    duration = t1 - t0;
  } else {
    console.log("WASM unsupported, expect delay during proof generation");
    const t0 = performance.now();

    await p.generate_work(config.salt, config.string, config.difficulty_factor);
    const t1 = performance.now();
    duration = t1 - t0;
  }
  return duration;
};

// credits: @jf-bastien on Stack Overflow
// https://stackoverflow.com/questions/47879864/how-can-i-check-if-a-browser-supports-webassembly
const WasmSupported = (() => {
  try {
    if (
      typeof WebAssembly === "object" &&
      typeof WebAssembly.instantiate === "function"
    ) {
      const module = new WebAssembly.Module(
        Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00)
      );
      if (module instanceof WebAssembly.Module)
        return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
    }
  } catch (e) {
    console.error(e);
  }
  return false;
})();

export default prove;
