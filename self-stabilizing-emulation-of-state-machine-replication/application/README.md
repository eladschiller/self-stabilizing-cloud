# Application

This directory contains the code for an instance of a single node. Make sure to change your current directory to the `application` directory. Type `cargo run -- --help` to se info on how to invoke the `application`.

The idea is that you create a hosts file with all the hosts you want to be part of the system. Then you copy this source code to all the hosts and specify the the above arguments to your liking. Doing it like this manually for each node is certainly possible, but it's not very convenient. Therefore I have the tools `local_starter` (for running multiple nodes on your own computer) and `remote_starter` (for running multiple nodes on different computers). Check out the readmes of those two for more information about them.

## Code overview

The entry point of the program is `main()` in the `main.rs` file. `main()` creates an instance of `Mediator` and spawns two threads for snapshot and write operations respectively.

`Mediator` is the core of the program and wires together the `Algorithms` in main and a `Communicator`. `Communicator` has a UDP socket that it receives from on a background thread. It also allows other structs to send UDP messages with it. Main holds the is the implementation of the algorithms (Failure detector and Binary consensus). `Main` and `Communicator` don't interact with each other directly. All interactions happen through the `Mediator`.
