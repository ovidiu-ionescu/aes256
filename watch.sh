#!/usr/bin/env bash

inotifywait -m src -e create -e move -e delete -e modify -e close_write |
    while read -r directory events filename; do
	echo "Rebuild documentation"
	./generate_docs.sh
    done
