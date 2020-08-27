
# Rusty Self-Stabilizing Snapshots

This project is an implementation of the paper Self-Stabilizing Snapshots, in Rust. However, a lot of the code can be reused for other algorithms as well - more about that later. This repository is similar to the (public) repository [Distributed-SWMR-registers](https://github.com/osklunds/Distributed-SWMR-registers), since ABD and snapshots are similar.


## How to run

1. [Install Rust](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Change directory to `local_starter` and type `cargo run 3 -w 1 -s 3 -v Algorithm3 -p`.

This will create three nodes on your local computer that all snapshot all the time. Only one node will write. Each node will print to the terminal in its own color when it starts/ends and a write or a snapshot operation. When a snapshot operation ends, the values of all nodes' registers will also be shown. So you see a sequence of write and snapshot operations and what the values of the registers were in the different places in that execution.


## Repository overview and how to reuse the code

A very brief overview of the repository follows below. A more detailed description, as well as how to reuse the various parts, can be found in `doc/Reuse.md`.

The `application` directory/crate contains the code for an instance of a snapshot node. On each computer you want to be part of this network, you run the code in this directory. More details are in `application/README.md`.

The `local_starter` directory/crate contains the code for a helper tool. `local_starter` automatically starts the user-supplied number of snapshot nodes on the local machine, to simplify testing of the code. Note that `local_starter` is purely for convenience. `application` works as a standalone program. More details are in `local_starter/README.md`.

The `remote_starter` directory/crate contains the code for another helper tool. `remote_starter` automatically starts snapshot nodes on remote machines via SSH. More details are in `remote_starter/README.md`.

The `evaluator` directory/crate contains the code for another helper tool. `evaluator` utilizes `remote_starter` to run different scenarios you define and aggregates the evaluation results.

The `commons` directory/crate contains code that is used by several of the above crates.

The `doc` directory contains some miscellaneous info that is also useful.


## Platform compatibility

|                                 | Linux | Mac | Windows |
|---------------------------------|-------|-----|---------|
| application                     | Yes   | Yes | Maybe   |
| local_starter                   | Yes   | Yes | No      |
| remote_starter: local computer  | Yes   | Yes | No      |
| remote_starter: remote computer | Yes   | Yes | No      |
| evaluator: local computer       | Yes   | Yes | No      |
| evaluator: remote computer      | Yes   | Yes | No      |


## License

This project uses code from [Distributed-SWMR-registers](https://github.com/osklunds/Distributed-SWMR-registers), which is licensed under the BSD-3-License. The corresponding license file is `3RD-PARTY-LICENSES/distributed_swmr_registers_license`.
