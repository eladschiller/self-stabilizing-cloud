#!/bin/python3.6

from pyeclib.ec_iface import ECDriver

ec_driver = ECDriver(k=1, m=5, ec_type='liberasurecode_rs_vand')
ec_driver.encode(b'foo')
