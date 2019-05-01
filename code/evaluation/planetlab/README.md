# PlanetLab Testbed Scripts

All scripts in this directory must be run from the root of this repository.

## Workflow

0. Make sure your working directory is the root of this repository.
1. Create a slice on PlanetLab, assign it to user(s) and add servers to it.
2. Populate `servers.txt`, `clients.txt` in `config/`. Duplicate lines will install duplicate instances.
3. Create test structure with `populate_steps.py`
4. Make sure the correct tests are symlinked to evaluation/planetlab/tests-enabled/
5. Install and start service.
6. Run tests.
7. Cleanup.

### Installation

```
$ config/populate_steps.py 10 "5, 10, 15, 20" 10 # For writers test
$ config/populate_steps.py "5, 10, 15, 20" 10 10 # For readers test
$ config/populate_steps.py 10 10 "5, 10, 15, 20" # For servers test
$ evaluation/planetlab/install_system.sh
$ evaluation/planetlab/servers_start.sh
```

### Tests

```
$ evaluation/planetlab/run_client_tests.sh <test case>
```

test cases: test-writers test-readers

### Cleanup

```
$ evaluation/planetlab/servers_teardown.sh
```
