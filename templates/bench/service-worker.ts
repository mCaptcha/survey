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

import { Bench, PoWConfig } from "./types";
import prove from "./prove";

const SALT = "674243647f1c355da8607a8cdda05120d79ca5d1af8b3b49359d056a0a82";
const PHRASE = "6e2a53dbc7d307970d7ba3c0000221722cb74f1c325137251ce8fa5c2240";

console.debug("worker registered");

onmessage = async (event) => {
  console.debug("message received at worker");
  const difficulty_factor = parseInt(event.data);
  const config: PoWConfig = {
    string: PHRASE,
    difficulty_factor,
    salt: SALT,
  };

  const duration = await prove(config);

  const msg: Bench = {
    difficulty: difficulty_factor,
    duration,
  };
  postMessage(msg);
};
