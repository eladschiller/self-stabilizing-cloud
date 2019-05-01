#!/bin/bash

if [ $# -eq 0 ]; then
    n=16
else
    n=$1
fi

i=2
while [ $i -le $n ]
do
  ./evaluation/perftest.sh $i
  sleep 1
  i=$((i*2))
done
