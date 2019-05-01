# Self-Stabilizing Communication Channel

This is the code for the self-stabilizing communication channel.

It serializes and deserializes data using the `pack` and `unpack` functions from
`packHelper.py`.

It multiplexes between concurrent clients after `chunks_size` bytes (configured
in `config/default.ini`).

It switches from UDP to TCP if the data object size is larger than 512 Bytes.

* `GossipProtocol.py` is the class which describes gossip messages.
* `ppProtocol.py` is the class that describes PingPong messages.
