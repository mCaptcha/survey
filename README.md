<div align="center">
<h1>mCaptcha Survey</h1>
<p>

**Performance statics survey runner**

</p>

[![Docker](https://img.shields.io/docker/pulls/mcaptcha/survey)](https://hub.docker.com/r/mcaptcha/survey)
[![status-badge](https://ci.batsense.net/api/badges/mCaptcha/survey/status.svg)](https://ci.batsense.net/mCaptcha/survey)


</div>

## Why

[mCaptcha](https://mcaptcha.org) is a
[proof-of-work](https://en.wikipedia.org/wiki/Proof_of_work) based
CAPTCHA system. Its effectiveness depends on an accurate and
time-relevant proof-of-work difficulty setting. If it is too high, it
could end up
[DoS-ing](https://en.wikipedia.org/wiki/Denial-of-service_attack) the
underlying service that it is supposed to protect and if it is too low,
the protection offered will be ineffective.

In order to select the right difficulty level, mCaptcha admins would
require knowledge about current performance benchmarks on a large
variety of devices that are currently on the internet.


## What

This program runs a mCaptcha benchmarks on user devices and collects
fully anonymous(only device statics are stored) performance statics,
that are transparently made available to everyone free of charge.
mCaptcha admins are kindly requested to refer to the benchmarks
published to fine-tune their CAPTCHA deployment.


## What data do you collect?

TODO: run program, record and share actual network traffic logs
