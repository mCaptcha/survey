#!/bin/python
# Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
import requests
import json

from creds import COOKIE


def add_campaign():
    """Add campaign"""
    url = "http://localhost:7000/admin/api/v1/campaign/add"
    payload = json.dumps(
        {
            "name": "test_1",
            "difficulties": [
                50000,
                100000,
                150000,
                200000,
                250000,
                300000,
                350000,
                400000,
                450000,
            ],
        }
    )
    headers = {"Content-Type": "application/json", "Cookie": COOKIE}

    response = requests.request("POST", url, headers=headers, data=payload)

    data = response.json()
    print("campaign ID: %s" % (data["campaign_id"]))


if __name__ == "__main__":
    add_campaign()
