#! /bin/bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ROOT_DIR=$SCRIPT_DIR/../../

cd $ROOT_DIR

stack run -- --port 3000 --host "127.0.0.1" --log-level="ALL" &
pid=$!

fswatch -r ./src | $SCRIPT_DIR/watch.sh $pid
