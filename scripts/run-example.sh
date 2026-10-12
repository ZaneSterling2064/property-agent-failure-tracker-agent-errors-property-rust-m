#!/bin/sh
set -eu

curl --fail-with-body \
  --request POST \
  --header 'content-type: application/json' \
  --data @examples/urgent_request.json \
  http://127.0.0.1:3000/agent/run

