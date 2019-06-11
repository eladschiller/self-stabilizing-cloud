# Self-stabilizing Byzantine fault-tolerant state machine replication
This folder contains the code produced during a Master's Thesis during Spring 2019 by [Axel Niklasson](https://github.com/axelniklasson) and [Therese Petersson](https://github.com/TheresePetersson).

Below is a (rather brief) explanation of the codebase and how it all ties together.

## BFTList
BFTList is the main implementation - think of it as the foundation for running a replicated state machine that is both self-stabilizing and Byzantine fault-tolerant. The state machine models a continously growing list which is implemented on the `master` branch. The two other branches are `no_op_automaton` (a never-growing list) and `failure_detector` which contains an event-driven failure detector. This makes more sense after having read [the original paper](https://research.chalmers.se/publication/503900).

## thor
Thor is used to bootstrap BFTList and makes sure that all needed config and environment variables are sent in. It also enables easy bootstrapping, meaning that the given number of nodes (`n`) are started as subprocesses in the current shell, eliminating the need for having `n` shells open just for running BFTList. BFTList should never be run without thor - think of it as a "BFTList runner".

## odin
Odin integrates with [PlanetLab EU](https://www.planet-lab.eu/) and its API in order to provide tooling for deploying BFTList to PlanetLab nodes. It performs healthchecks on the nodes, downloads fresh copies of both BFTList and Thor (since thor is needed to run BFTList) and starts them up, all over SSH. Think of this as the tool to use when deploying to PlanetLab.

## heimdall
Heimdall handles metrics. It contains a docker-compose file for starting up two containers, one Prometheus container for aggregating metrics and one Grafana container for visualizing these metrics.