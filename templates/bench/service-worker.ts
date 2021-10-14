/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

import { gen_pow } from "mcaptcha-browser";
import { Bench } from "./types";

type PoWConfig = {
  string: string;
  difficulty_factor: number;
  salt: string;
};

const SALT = "674243647f1c355da8607a8cdda05120d79ca5d1af8b3b49359d056a0a82";
const PHRASE = "6e2a53dbc7d307970d7ba3c0000221722cb74f1c325137251ce8fa5c2240";

const config: PoWConfig = {
  string: PHRASE,
  difficulty_factor: 1,
  salt: SALT,
};

console.debug("worker registered");

onmessage = function (event) {
  console.debug("message received at worker");
  const difficulty_factor = parseInt(event.data);
  config.difficulty_factor = difficulty_factor;

  const t0 = performance.now();
  gen_pow(config.salt, config.string, config.difficulty_factor);
  const t1 = performance.now();
  const duration = t1 - t0;

  const msg: Bench = {
    difficulty: difficulty_factor,
    duration,
  };
  postMessage(msg);
};
