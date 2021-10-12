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
import {gen_pow} from 'mcaptcha-browser';
import {Perf} from './types';

type PoWConfig = {
  string: string;
  difficulty_factor: number;
  salt: string;
};

const SALT = '674243647f1c355da8607a8cdda05120d79ca5d1af8b3b49359d056a0a82';
const PHRASE = '6e2a53dbc7d307970d7ba3c0000221722cb74f1c325137251ce8fa5c2240';

const config: PoWConfig = {
  string: PHRASE,
  difficulty_factor: 1,
  salt: SALT,
};

console.debug('worker registered');

onmessage = function(event) {
  console.debug('message received at worker');
  let difficulty_factor = parseInt(event.data);
  config.difficulty_factor = difficulty_factor;

  const t0 = performance.now();
  gen_pow(config.salt, config.string, config.difficulty_factor);
  const t1 = performance.now();
  const time = t1 - t0;

  let msg: Perf = {
    difficulty: difficulty_factor,
    time: time,
  };
  postMessage(msg);
};
