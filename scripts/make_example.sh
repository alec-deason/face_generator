#!/bin/sh

exec 2>/dev/null

FILE_NAME=`date +%Y_%m_%d_%k_%M`.txt

TMP_FILE=$(mktemp "${TMPDIR:-/tmp/}$(basename $0).XXXXXXXXXXXX")


if cargo run  > $TMP_FILE; then
    cp $TMP_FILE examples/$FILE_NAME
fi

rm $TMP_FILE
