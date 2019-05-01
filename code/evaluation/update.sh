#!/bin/bash

if [ $# -eq 0 ]; then
    n=4
else
    n=$(($1-1))
fi

# Update files
echo "Updating files..."
for i in $( seq 0 $n ); do
    docker cp . "c$i:/opt/project"
done;


