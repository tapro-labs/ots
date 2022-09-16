#! /bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=$SCRIPT_DIR/../../

cd $ROOT_DIR

pid=$1
while read; do
  if ! [[ -z $pid ]]; then
    kill $pid
  fi

  stack run -- --port 3000 --host "127.0.0.1" --log-level="ALL" &

  pid=$!
done
