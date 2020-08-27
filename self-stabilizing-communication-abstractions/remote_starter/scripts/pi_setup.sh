#!/bin/bash
sudo apt install libtool autoconf autogen build-essential cmake -y
sudo apt upgrade
curl https://sh.rustup.rs -sSf > rustup.sh
chmod +x rustup.sh
./rustup.sh -y
