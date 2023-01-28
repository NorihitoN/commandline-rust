#!/usr/bin/env bash
# tr ':' '\n' <<< "$PATH"

set -u 
# https://www.gnu.org/software/bash/manual/html_node/The-Set-Builtin.html

ROOT="tests/inputs"
OUT_DIR="tests/expected"

[ ! -d "$OUT_DIR" ] && mkdir -p "$OUT_DIR"
# same as follow code
# if [ ! -d "$OUT_DIR" ]; then
#     mkdir "$OUT_DIR"
# fi

EMPTY="$ROOT/empty.txt"
FOX="$ROOT/fox.txt"
SPYDERS="$ROOT/spiders.txt"
BUSTLE="$ROOT/the-bustle.txt"
ALL="$EMPTY $FOX $SPYDERS $BUSTLE"

for FILE in $ALL; do
    BASENAME=$(basename "$FILE")
    cat    $FILE > ${OUT_DIR}/${BASENAME}.out
    cat -n $FILE > ${OUT_DIR}/${BASENAME}.n.out
    cat -b $FILE > ${OUT_DIR}/${BASENAME}.b.out
done

cat    $ALL > $OUT_DIR/all.out
cat -n $ALL > $OUT_DIR/all.n.out
cat -b $ALL > $OUT_DIR/all.b.out

cat < $BUSTLE > $OUT_DIR/$(basename $BUSTLE).stdin.out
cat -n < $BUSTLE > $OUT_DIR/$(basename $BUSTLE).n.stdin.out
cat -b < $BUSTLE > $OUT_DIR/$(basename $BUSTLE).b.stdin.out