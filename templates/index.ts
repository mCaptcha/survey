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
import {Perf} from './types';

const FACTOR = 500000;
const worker = new Worker('bench.js');
const res: Array<Perf> = [];
const stats = document.getElementById('stats');

const addResult = (perf: Perf) => {
  const row = document.createElement('tr');
  row.className = 'data';
  const diff = document.createElement('td');
  diff.innerHTML = perf.difficulty.toString();
  const duration = document.createElement('td');
  duration.innerHTML = perf.time.toString();

  row.appendChild(diff);
  row.appendChild(duration);

  stats.appendChild(row);

  res.push(perf);
};

const addDeviceInfo = () => {
  const INFO = {
    threads: window.navigator.hardwareConcurrency,
    oscup: window.navigator.userAgent,
  };

  console.log(res);
  console.log(INFO);

  const element = document.createElement('div');
  const ua = document.createElement('b');
  ua.innerText = 'User Agent: ';
  const os = document.createTextNode(`${INFO.oscup}`);

  const threads = document.createElement('b');
  threads.innerText = 'Hardware concurrency: ';
  const threadsText = document.createTextNode(`${INFO.threads}`);

  element.appendChild(ua);
  element.appendChild(os);
  element.appendChild(document.createElement('br'));
  element.appendChild(threads);
  element.appendChild(threadsText);

  document.getElementById('device-info').appendChild(element);
};

const finished = () => {
  const s = document.getElementById('status');
  s.innerHTML = 'Benchmark finished';
};

const run = (e: Event) => {
  e.preventDefault();
  document.getElementById('pre-bench').style.display = 'none';
  document.getElementById('bench').style.display = 'flex';

  const iterations = 9;

  const counterElement = document.getElementById('counter');
  counterElement.innerText = `${iterations} more to go`;

  worker.onmessage = (event: MessageEvent) => {
    let data: Perf = event.data;
    addResult(data);
    if (res.length == iterations) {
      finished();
      counterElement.innerText = `All Done!`;
    } else {
      counterElement.innerText = `${iterations - res.length} more to go`;
    }
  };

  for (let i = 1; i <= iterations; i++) {
    let difficulty_factor = i * FACTOR;
    worker.postMessage(difficulty_factor);
  }

  addDeviceInfo();
};

document.getElementById('start').addEventListener('click', e => run(e));
