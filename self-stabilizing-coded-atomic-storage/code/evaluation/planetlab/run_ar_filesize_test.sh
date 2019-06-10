#!/bin/bash

rounds=50

opts="StrictHostKeyChecking=no"
ssh_key=~/.ssh/planetlab_rsa
slice=chalmersple_casss2
tests=evaluation/planetlab/tests_enabled/*
test_case_dir=config/tests/$1/*
config="/home/$slice/casss/config/autogen.ini"

for step in $test_case_dir; do
  scenario=$(echo "$step" | sed 's/.*\/\(step.*\)/\1/')

  for file in $step/*; do
    if [[ $file = *'readers.txt' ]]; then
      r=$file
      readers=$(cat $r | sort)
    elif [[ $file = *'writers.txt' ]]; then
      w=$file
      writers=$(cat $w | sort)
    fi
  done

  for t in $tests; do
    if [[ $t == *'/filesize_ar_test.py' ]]; then
      m=$(echo $t | sed 's/\//./g')
      module=${m%.py}
      for writer in $writers; do
        ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $writer \
          "cd ~/casss/; python3.6 -O -m $module $rounds $config $step" &
      done
      for reader in $readers; do
        ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $reader \
          "cd ~/casss/; python3.6 -O -m $module $rounds $config $step" &
      done
    fi
  done

  echo "Tests started!"
  wait
  echo "Tests finished!"

  echo "Fetching results..."
  mkdir -p $step
  for writer in $writers; do
    rsync -aPz -e "ssh -o $opts -i $ssh_key -l $slice"\
      $writer:~/results/$scenario* $step
  done
  for reader in $readers; do
    rsync -aPz -e "ssh -o $opts -i $ssh_key -l $slice"\
      $reader:~/results/$scenario* $step
  done

  python3.6 -m evaluation.planetlab.summary $step

done

echo "All tests done!"
# echo "Cleaning up clients..."
#evaluation/planetlab/cleanup_clients.sh
echo "DONE"
