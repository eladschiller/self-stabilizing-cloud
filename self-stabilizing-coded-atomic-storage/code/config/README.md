# Configuration

This directory is almost misc, but not quite.

* `server.txt` and `clients.txt` are newline separated lists of hostnames. For
  CAS\_ZMQ, it instead needs to be IP addresses.
* `create.py` is used to create a configuration file (`*.ini`), via an
  interactive prompt.
* `default.ini` holds the configuration parameters used when automatically
  creating configuration files (`autogen.ini`), so that needs to not have `n` or
  any servers in it.
* Unfortunately, `populate_steps.py` is used to create the experiment scenarios
  in `./tests/`. It should really be somewhere else than here, since it has
  nothing to do with configuration.
* `util/` also has nothing to do with configuration. But it has a lot of useful
  things in it. The scripts in `util/` are used to find out which PlanetLab
  servers works for our purposes.
