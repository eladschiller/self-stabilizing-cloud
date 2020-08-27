
# Evaluator - A helper utilty that gathers evaluation results and aggregates them

This directory contains the code for a helper utility that gathers evaluation results from running the code on multiple remote machines and also aggregates the results. Make sure to change your current directory to the `evaluator` directory. Type `cargo run -- --help` to see how to invoke `evaluator`. Evaluator has the three subcommands `aggregate`, `gather` and `install`.

The indented workflow is as follows:

1. Create the hosts file.
2. Use the `install` command to install Rust and the source code on the hosts.
3. Create the scenario file.
4. Use the `gather` command to run all scenarios.
5. Stash away the result file if you want each scenario to be run more than once.
6. Repeat step 4 and 5 until you have the number of results you want.
7. Define programtically (you have to change the source code) functions on the result data, for example, the average write latency, that you are intersted in.
8. Use the `aggregate` command to run your code on the result data.

**IMPORTANT: READ ALL NOTES BELOW. ESPECIALLY REMOTE_STARTER MENTIONS THINGS YOU DEFINITELY SHOULD READ.**

## Notes for usage

Check the notes for usage of `remote_starter`. Those notes apply here as well. In addition there are the notes below:

### Scenario file

An example of what the scenario file should look like is found in `scenarios_example.txt` and is also found below:

```
15,0,1,Algorithm1,0
15,0,5,Algorithm1,0
15,0,10,Algorithm1,0
15,0,15,Algorithm1,0
```

From left to right: number of nodes, number of snapshotters, number of writers, variant, delta. Each line is a scenario. Unless the variant is 4, the value of delta doesn't matter.


### Result files

Evaluator will download files with names `node001.eval`, `node002.eval`, ... from the remote hosts. You don't need to save or care about these files.

The end result is stored in `results.eval`. It is a json-serialization of a `HashMap`. You should't modify it. Keep this file though.

