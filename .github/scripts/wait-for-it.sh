#!/bin/bash

set -e

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 host:port" >&2
  exit 1
fi

TIME=1
MAX_TRIES=10

while [ $MAX_TRIES -gt 0 ]; do
  if  curl -sS $1 > /dev/null 2>&1; then
    exit 0
  fi
  sleep $TIME
  TIME=$((TIME*2))
  MAX_TRIES=$((MAX_TRIES-1))
done
exit 1
