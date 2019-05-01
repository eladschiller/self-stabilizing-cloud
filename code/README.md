**Authors:**

Robert Gustafsson <robg@student.chalmers.se>

Andreas Lindhé <andreas@lindhe.io>


# Self-Stabilizing Services for Emulating Distributed Shared Memory on Message Passing Platforms

This is the outcome of our master's thesis project, during the spring of 2018.
We are the first ones to implement and practically evaluate the self-stabilizing
version of _Coded Atomic Storage_ (CAS) by
[Dolev, Petig and Schiller](https://arxiv.org/abs/1806.03498).


## License

This project is licensed under the MIT license.
Please see the LICENSE file.
The copyright belongs to the authors.


## Installation

The project was exclusively developed on and for Linux, but might run on other
system too.
If you find important points for a specific platform, please tell us.

**IMPORTANT** -- To get access to NS-3, make sure to clone this repository using
the following command:
```
$ git clone --recursive git@bitbucket.org:selfStabilizingAtomicStorage/datx05-code.git
```


### Dependencies

* `bash`
* `python3.6`
* `liberasurecode`
* `docker`
* `rsync`
* `ssh`
* `tunctl`
* (NS-3)

**Install required python modules:**
```
$ pip3.6 install -r requirements.txt
```


### Needed for the CAS\_ZMQ version:

* `python3.6`
* `zeromq`
* `pyzmq`


## Repository Structure

There is a method to our madness, at least sometimes.
It would be good to clean up things more, but this is a brief overview of what's
where in this repository.
More detailed README files are found in each of the subdirectories, but this is
the gists of it:


### Code

**Terms:**
* CASSS is the self-stabilizing version of CAS
* CAS\_ZMQ is an implementation of CAS which does not use any self-stabilizing techniques
* AR stands for Atomic Register
* Atomic Register is an incorrect name for the SIMPLE algorithm by Georgiou
* SIMPLE is a MWMR version of ABD

#### Directories
* `atomic_regsiter/`: code for the SIMPLE algorithm (MWMR ABD)
* `cas/`: code for CAS and CASSS algorithms
* `channel/`: code for the self-stabilizing communication channel
* `gossip/`: code for the self-stabilizing gossiping service
* `quorum/`: code for the self-stabilizing quorum system
* `register/`: code for the module which handles storage in CAS

#### Files
* `ARclient.py`: a class implementing the SIMPLE algorithm's client protocol
* `ARserver.py`: a class implementing the SIMPLE algorithm's server protocol
* `autostart.sh`: bootstrap script for starting SIMPLE server on PlanetLab
* `cas_zmq_autostart.sh`: bootstrap script for starting CAS\_ZMQ server on PlanetLab
* `client.py`: a class implementing the CASSS algorithm's client protocol
* `server.py`: a class implementing the CASSS algorithm's server protocol
* `run_client.py`: an application running an instance of a CASSS client
* `start_multiple_servers.py`: an application spinning up multiple local instances of the CASSS server
* `start_cas_zmq_servers.py`:  an application spinning up multiple local instances of the CAS_ZMQ server


### Configuration, tests, evaluation, misc.

#### Directories
* `config/`: is meant to hold configuration files and related scripts.
  But accidentally, it also includes a lot of other things, mainly related to
  evaluation and debugging.
* `evaluation/`: scripts for evaluation
* `test/`: test to find out if basic functionality of the system works

#### Files
* `Dockerfile`: configuration file used for test bed (and potentially deployment)
* `requirements.py`: required dependencies for pip (and/or Docker) to install


### Usage

Before running anything, make sure that `config/servers.txt` and
`config/clients.txt` are populated with the correct hosts.
Also, make sure that `config/default.ini` is properly configured.


#### Locally

1: Create a configuration file:
```
$ cd config
$ ./create.py
$ cd ..
```

2: Start the servers and run the client:
```
$ python -u ./start_multiple_servers.py ./config/config.ini
$ python -O ./run_client.py ./config/config.ini
```


#### Test Bed

To perform a throughput test, just run:
```
$ ./evaluation/perftest.sh
```

Otherwise:

1: Start NS-3
```
$ sudo ./waf --run "tap-csma-virtual-machine --n=4"
```

2: Create `n` servers (`n=4` by default):
```
$ ./evaluation/create_new.sh 4
```

3: Run a client
```
$ python -O ./run_client.py ./config/autogen.ini
```

4: Tear down the test bed

```
$ ./evaluation/remove_old.sh
```


#### Evaluation Platform

We used PlanetLab to build an evaluation platform.
The relevant scripts for this are mostly located in `evaluation/planetlab/`.

**Before running, make sure of the following:**

* All experiments you want to run are symlinked from `evaluation/planetlab/tests_available/` into `evaluation/planetlab/tests_enabled/`.
* `config/servers.txt` and `config/clients.txt` are populated with the correct
  PlanetLab nodes.
* Your PlanetLab account has access to the slices, and you have uploaded your
  public SSH key to that account.
* The private SSH key to the slice is available in ~/.ssh/planetlab_rsa

**TODO:** it would be really nice to have slice name as a configurable
parameter, instead of having to change it in a billion files every time it
switches. I made a new branch with a single commit, which I changed to when I
wanted to switch slice.

1: Install the system:
```
$ evaluation/planetlab/install_system.sh
```

2: Start servers:
```
$ evaluation/planetlab/servers_start.sh
```

3: Check that the service seems to work:
```
$ python -m evaluation.planetlab.servers_init ./config/autogen.ini
```

4: Prepare test (e.g. server scalability):
```
$ config/populate_steps.py 10 10 "5, 10, 15, 20"
```

5: Run test
```
$ evaluation/planetlab/run_client_tests.sh test-servers
```

6: Cleanup after yourself:
```
$ ./evaluation/planetlab/cleanup_clients.sh && ./evaluation/planetlab/servers_teardown.sh
```

## Unit tests:

**One specific test:**
```
$ python -m unittest register/tests/test_Register.py
```

**Run all tests:**
```
$ python -m unittest
```

### Code Style

* Line width: 80 characters
* Indentation: 2 spaces
* Use verbose variable names whenever possible
* All filenames and module names should be in `snake_case`.
* All class names should be in `CamelCase`.
* Documentation should be in
  [Google Style docstrings](https://sphinxcontrib-napoleon.readthedocs.io/en/latest/example_google.html).
* All scripts should start with a `#!` defining the interpretor to use (e.g.,
  `#!/bin/python3.6`)
* Try to keep commits atomic
* Use roughly [this Git branching model](https://nvie.com/posts/a-successful-git-branching-model/)
