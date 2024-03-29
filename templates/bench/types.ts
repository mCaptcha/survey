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

export type Bench = {
  difficulty: number;
  duration: number;
};

export type Submission = {
  device_user_provided: String;
  device_software_recognised: String;
  threads: number;
  benches: Array<Bench>;
  submission_type: SubmissionType;
};

export type SubmissionProof = {
  token: String;
  proof: String;
};

export type BenchConfig = {
  difficulties: Array<number>;
};

export type PoWConfig = {
  string: string;
  difficulty_factor: number;
  salt: string;
};

export enum SubmissionType   {
  wasm = "wasm",
  js = "js",
}
