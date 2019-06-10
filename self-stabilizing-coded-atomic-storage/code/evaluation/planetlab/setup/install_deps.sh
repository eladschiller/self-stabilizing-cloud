#!/bin/bash

sudo dnf install -y python3-devel python36 redhat-rpm-config liberasurecode-devel python3-lxml python3-zmq
sudo python3.6 ~/casss/evaluation/planetlab/get-pip.py
sudo pip3.6 install -r ~/casss/requirements.txt
