#!/bin/bash

read -n 1 -p "Remember to update servers.txt and clients.txt! Press enter to continue..."
./evaluation/planetlab/setup/create_config.sh
./evaluation/planetlab/setup/copy_to_planetlab.sh
./evaluation/planetlab/setup/install.sh
