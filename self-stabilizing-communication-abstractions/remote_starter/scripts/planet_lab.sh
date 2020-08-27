#!/bin/bash
sudo pkill -f rusty
sudo yum group install "Development Tools" -y

curl https://sh.rustup.rs -sSf > rustup.sh
chmod +x rustup.sh
./rustup.sh -y
