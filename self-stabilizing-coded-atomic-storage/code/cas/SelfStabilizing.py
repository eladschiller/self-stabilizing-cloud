#!/bin/python3.6
# -*- coding: utf-8 -*-
#
# MIT License
#
# Copyright (c) 2018 Robert Gustafsson
# Copyright (c) 2018 Andreas Lindhé
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

import struct
import sys
import asyncio
from collections import deque
from .IncNbrHelper import IncNbrHelper
from channel.GossipProtocol import GossipMessage
from channel.ppProtocol import PingPongMessage
from register.record import Record
from register.register import Register
from .proposal import Proposal as Prp
from itertools import combinations


class Server:
  """ The event handlers for the server in Algorithm 2 """

  def __init__(self, uid, quorum, max_clients, delta, queue_size, n,
storage_location="./.storage/", gossip_freq=1):
    self.uid = uid.encode()
    self.t_top = 2147483647 # max value for signed int for struct format 'i'
    self.queue_size = queue_size
    self.inc_nbrs = deque(maxlen=queue_size)
    self.gossip_freq = gossip_freq
    self.gsp_freq = gossip_freq
    # Quorum size:
    self.quorum = quorum
    # Initialize with an empty register S
    self.S = Register(max_clients, delta, storage_location)
    # Received gossip messages. Key is UID and value is tag
    self.pre = {}
    self.fin = {}
    self.FIN = {}
    ######### Wrap Around (Global Reset) #########
    self.n = n
    # Algorithm variables:
    self.dflt_prp = Prp(0, None)
    self.config = [self.uid] # List of uid
    self.prp = {self.uid: Prp(0, None)} # {uid: Proposal} or {uid: None}
    self.all = {self.uid: False} # {uid: bool}
    self.echo_answers = {}
    self.all_seen_processors = set()

  def counter_query(self, sender):
    """ Reply to queries with current counter value """
    if (IncNbrHelper.max_reached(self.inc_nbrs, 100)):
      return None
    inc_nbr = IncNbrHelper.get(self.inc_nbrs, sender)
    cntr = struct.pack("i", inc_nbr)
    return (None, cntr, None)

  def set_counter(self, sender, new_value):
    """ Replace counter value if larger """
    IncNbrHelper.set(self.inc_nbrs, sender, new_value)
    return (None, None, None)

  def read_query(self):
    """ Reply to read query arrival event, from pj's client to pi's server.

    Returns:
      tuple (t, None, 'qry') where t is max_phase of ['fin', 'FIN']
    """
    max_tag = self.S.max_phase(['pre', 'fin', 'FIN'])
    if max_tag[0] >= self.t_top:
      return None
    else:
      return (self.S.max_phase(['fin', 'FIN']), None, 'qry')

  def write_query(self):
    """ Reply to write query arrival event, from pj's client to pi's server.

    Returns:
      tuple (t, None, 'qry') where t is max_phase of ['pre', 'fin', 'FIN']
    """
    max_tag = self.S.max_phase(['pre', 'fin', 'FIN'])
    if max_tag[0] >= self.t_top:
      return None
    else:
      return (max_tag, None, 'qry')

  async def pre_write(self, t, w):
    """ Reply to pre-write arrival event, from pj's writer to pi's server.

    Args:
      t (tuple): tag (Sequence number, UID)
      w (Byte): coded element
    Returns:
      (t, None, 'pre')
    """
    await self.S.update_phase(t, w, 'pre')
    return (t, None, 'pre')

  async def read_finalize(self, t, d):
    """ Update register and return a reply, for fin or FIN pp-arrival event.

    Args:
      t (tuple): tag (Sequence number, UID)
      d (string): phase 'fin' or 'FIN'
    Returns:
      tuple: (t, w, d) where w is the coded element if it exists in the record,
      or None otherwise.
    """
    await self.S.update_phase(t, None, d)
    w = await self.S.fetch(t)
    return (t, w, d)

  async def write_finalize(self, t, d):
    """ Update register and return a reply, for fin or FIN pp-arrival event.

    Args:
      t (tuple): tag (Sequence number, UID)
      d (string): phase 'fin' or 'FIN'
    Returns:
      tuple: (t, None, d)
    """
    if not ((t not in self.S.register) and (t[0] == self.t_top)):
      await self.S.update_phase(t, None, d)
    return (t, None, d)

  async def gossip_departure(self, k):
    """ Send the latest values open gossip depature event to server k """
    tag_tuple = self.S.tag_tuple()
    cntr = IncNbrHelper.to_list(self.inc_nbrs) if len(self.inc_nbrs) else None
    prp = tuple(self.prp[self.uid]) if self.prp[self.uid] else None
    msg_all = self.all[self.uid]
    echo_prp = tuple(self.prp[k]) if k in self.prp and self.prp[k] else None
    echo_all = self.all[k] if k in self.all else False
    echo = (echo_prp, echo_all)
    gossip_obj = GossipMessage(tag_tuple, cntr, prp, msg_all, echo)
    data = gossip_obj.get_bytes()
    return (not self.enable_reset(), data)

  async def gossip_arrival(self, k, pre, fin, FIN, prp, msg_all, echo, inc_nbrs):
    """ Reply to gossip arrival event, from pj's server to pi's server.

    Updates the class variables pre, fin and FIN to include the content of the
    pre, fin and FIN received via a gossip message from server with uid k.

    Args:
      k (string): the UID of the server from which we received a gossip message
      pre (tuple): the pre[k] tag received in gossip[k]
      fin (tuple): the fin[k] tag received in gossip[k]
      FIN (tuple): the FIN[k] tag received in gossip[k]
    Returns:
      tag_tuple(): (max_all, max_finFIN, max_FIN)
    """
    

    # Update counter
    if inc_nbrs:
      IncNbrHelper.merge(self.inc_nbrs, inc_nbrs)

    #Check for max tag
    max_tag = self.S.max_phase(['pre', 'fin', 'FIN'])
    if (max_tag[0] >= self.t_top and self.stabilized()):
      self.propose(max_tag)
    elif (self.and_every(self.all_same)):
      # We need to store the received tag values, for the implicit FIN to work
      (self.pre[k], self.fin[k], self.FIN[k]) = (pre, fin, FIN)
      i = self.uid
      # pre
      self.pre[i] = max( pre, fin, FIN, self.S.max_phase(['pre', 'fin', 'FIN']) )
      await self.S.update_phase(self.pre[i], None, 'pre')
      # fin
      self.fin[i] = max( fin, FIN, self.S.max_phase(['fin', 'FIN']) )
      await self.S.update_phase(self.fin[i], None, 'fin')
      #FIN
      implicitFinalized = []
      fin_tags = [t for t in self.fin.values() if t == fin]
      if len(fin_tags) >= self.quorum:
        implicitFinalized = [fin]
      self.FIN[i] = max( FIN, self.S.max_phase(['FIN']), *implicitFinalized )
      await self.S.update_phase(self.FIN[i], None, 'FIN')
      # Remove not relevant records
      self.S.relevant()

    # Run wrap around algorithm
    self.run_global_reset(k, prp, msg_all, echo)

    #Limit gossip frequency
    await asyncio.sleep(self.gsp_freq)

  def stabilized(self):
    """ Check if the records have yet stabilized """
    max_tuple = self.S.tag_tuple()
    max_pre = max_tuple[0]
    max_fin = max_tuple[1]
    max_FIN = max_tuple[2]
    tag_tuple_stab = (max_fin == max_FIN) and (max_pre >= max_fin)
    pre_set = set(self.pre.values())
    pre_stab = (len(pre_set) == 1) and (max_pre in pre_set)
    fin_set = set(self.fin.values())
    fin_stab = (len(fin_set) == 1) and (max_fin in fin_set)
    FIN_set = set(self.FIN.values())
    FIN_stab = (len(FIN_set) == 1) and (max_FIN in FIN_set)
    return tag_tuple_stab and pre_stab and fin_stab and FIN_stab


  def get_tag_tuple(self):
    """ return largest tag tuples """
    return self.S.tag_tuple()

  def get_counter(self):
    """ return list of incarnation numbers if there exist any """
    return IncNbrHelper.to_list(self.inc_nbrs) if len(self.inc_nbrs) else None

######### Wrap Around (Global Reset) #########

  def run_global_reset(self, k, prp, msg_all, echo):
    """ Function to run global reset algorithm if everyone has initially
        communicated """
    if k not in self.config:
      self.config.append(k)
    self.prp[k] = Prp(*prp) if prp else None
    self.all[k] = msg_all
    if echo[0]:
      self.echo_answers[k] = (Prp(*echo[0]), echo[1])
    else:
      self.echo_answers[k] = (None, echo[1])
    if len(self.config) == self.n:
      self.global_reset()

  def global_reset(self):
    """ Global reset algorithm's main loop """
    if self.enable_reset():
      self.gsp_freq = self.gossip_freq
    max_tag = self.S.max_phase(['pre', 'fin', 'FIN'])
    if max_tag[0] >= self.t_top:
      self.gsp_freq = 0 # increase communication frequency during reset
    # Main loop
    uid = self.uid
    for k in self.config:
      if k != self.uid:
        if (self.all[k]):
          self.all_seen_processors.add(k)
    if self.transient_fault():
      if __debug__:
        print("Transient fault detected!")
      self.prp_set(None)
    # Update all[i]:
    if (self.prp[uid] == None and self.all[uid]):
      if __debug__:
        print("Changing to dfltPrp!")
      self.prp[uid] = self.dflt_prp
    self.all[uid] = self.and_every(self.echo_no_all)
    if self.no_default_no_bot():
      self.gsp_freq = 0
      self.prp[uid] = self.max_prp()
      self.all[uid] = self.and_every(self.echo_no_all)
      if self.all_seen() and self.and_every(self.echo):
        (self.prp[uid], self.all[uid]) = self.increment(self.prp[uid])
        self.all_seen_processors = set()
      if self.prp[uid].phase == 2:
        self.local_reset(self.prp[uid].tag)
        if __debug__:
          print("LOCAL RESET")

  def propose(self, tag):
    """ Proposes to reset register to only hold the Record with tag tag.

    Args:
      tag (tuple): tag to use as new ground truth
    """
    if self.enable_reset():
      self.prp[self.uid] = Prp(1, tag)
      self.all[self.uid] = False

  def enable_reset(self):
    """ Blocks proposal if ongoing proposal.

    Returns:
      bool: True if good to go, False otherwise
    """
    for k in self.config:
      if self.prp[k] == None \
          or ((self.prp[k], self.all[k]) != (self.dflt_prp, True)):
        return False
    return True

  def prp_set(self, val):
    """ Sets prp[k] to val for each k in self.config. """
    for k in self.config:
      self.prp[k] = val
      self.all[k] = False
    return

  def mod_max(self):
    """ Return the maximum phase of all proposals, in a modulus manner.

    Returns:
      int: a phase represented by 0, 1 or 2
    """
    i = self.uid
    phs = set([ self.prp[k].phase for k in self.config if self.prp[k] ])
    if phs == set([0, 1]):
      return max(phs)
    else:
      return self.prp[self.uid].phase

  def degree(self, k):
    """ Returns the degree of proposal k.

    Args:
      k (uid): which proposal to look at.
    Returns:
      int: the degree of prp[k]
    """
    if not self.prp[k]:
      return 0
    return 2*self.prp[k].phase + (1 if self.my_all(k) else 0)

  def corr_deg(self, k, l):
    """ Returns whether the degrees of prp[k] and prp[l] correlates.

    Args:
      k, l (uid): The proposals to look at.
    Returns:
      bool: True if they correlate, False otherwise.
    """
    a = self.degree(k)
    b = self.degree(l)
    correlating_degrees = []
    for i in range(6):
      correlating_degrees.append(set([i, i]))
      correlating_degrees.append(set([i, (i+1)%6]))
      correlating_degrees.append(set([i, (i+2)%6]))
    return set([a, b]) in correlating_degrees

  def max_prp(self):
    """ Returns the maximal proposal.

    The loop does double duty: it collects a list of proposed tags, but will
    also return if phases are not synchronized.

    Returns:
      tuple: proposal
    """
    i = self.uid
    prp_tags = []
    for k in self.config:
      if self.prp[k]:
        if self.prp[k].tag:
          prp_tags.append(self.prp[k].tag)
        if ((self.degree(k) - self.degree(i))%6 not in set([0, 1])):
          return self.prp[i]
      else:
        # If there was a None proposal, let's not progress
        return self.prp[i]
    m_max = self.mod_max()
    if prp_tags:
      if m_max:
        return Prp(m_max, max(prp_tags))
      else:
        return Prp(m_max, None)
    else:
      return Prp(m_max, None)

  def my_all(self, k):
    """ The myAll macro.

    Returns:
      bool: processor k is in all_seen_processors or all[k]==True
    """
    if self.all[k]:
      return True
    for p in self.all_seen_processors:
      if not self.prp[k] or not self.prp[p]:
        return False
      if (self.prp[p].phase == (self.prp[k].phase+1)%3) and \
self.prp[k].phase != self.prp[p].phase:
        return True
    return False

  def echo_no_all(self, k):
    """ Checks the echoed proposal from processor k.

    Args:
      k (uid): check last echo message from processor k
    Return
      bool: True if my proposal is the echoed proposal from processor k
    """
    return (self.prp[self.uid] == self.echo_answers[k][0]) and \
(self.larger_or_equal(k))
   
  def all_same(self, k):
    """ Test so that everyone has dfltPrp and that everyone else seen it """
    prps = set([self.prp[k] for k in self.config])
    if (None in prps):
      return False
    return self.enable_reset() and (self.all[self.uid] == self.my_all(k))

  def larger_or_equal(self, k):
    """ Returns true if k is at the same phase or one ahead of i """
    if not self.prp[k] or not self.prp[self.uid]:
      return True
    if self.prp[k].phase == (self.prp[self.uid].phase+1)%3:
      return True
    elif self.prp[k].phase == self.prp[self.uid].phase:
      return True
    else:
      return False
      

  def echo(self, k):
    """ Checks if my proposal and my_all variable are echoed by processor k.

    Args:
      k (uid): the processor which echo to check
    Returns:
      bool: True if my proposal and my_all are what's echoed back by processor
        k, false otherwise
    """
    return ((self.prp[self.uid], self.all[self.uid]) == self.echo_answers[k]) \
and self.larger_or_equal(k)

  def increment(self, proposal):
    """ Returns the appropriate incremented new proposal based on its phase.

    Args:
      prp (tuple): the proposal to increment.
    Returns:
      tuple: Proposal which is (2, prp) if prp.phase == 1, or self.dflt_prp
      otherwise.
    """
    if proposal.phase == 1:
      return (Prp(2, proposal.tag), False)
    elif proposal.phase == 2:
       return (self.dflt_prp, False)
    else:
       return (self.prp[self.uid], self.all[self.uid])

  def all_seen(self):
    """ Reports if everyone have seen my proposal.

    Returns:
      bool: True if all is True and all active processors are in all_seen_processors
    """
    return self.all[self.uid] \
        and set(self.config) <= self.all_seen_processors | set([self.uid])

  def proposal_set(self):
    """ If anyone is in phase 2, collect all non-bot proposals.

    Set of proposed tags, if there is an l in config for which prp[l]=(2, *)

    Return:
      list: list of all proposals, or empty list
    """
    proposal_phases = [self.prp[k].phase if self.prp[k] else None for k in self.config]
    if 2 in proposal_phases:
      return set([self.prp[k].tag for k in self.config if self.prp[k] and self.prp[k].tag ])
    else:
      return []

  def and_every(self, macro):
    """ Logical AND of every return value for macro[k] such that k is in config.

    Args:
      macro (function): macro must refer to a function which can take argument k
        and return a bool.
    Returns:
      bool: True if all macro[k]==True, False otherwise.
    """
    for k in self.config:
      if k != self.uid:
        if not macro(k):
          return False
    return True

  def transient_fault(self):
    """ Checks for all possible transient faults.

    Returns:
      bool: True if there is a transient fault detected, False otherwise.
    """
    if not self.prp[self.uid]:
      return False
    # (∃pk ∈ config : prp[k] = ⊥)
    is_bot_prp = [k for k in self.config if self.prp[k] == None]
    # (prp[i] != {dfltPrp})
    prp_other_than_dflt = self.prp[self.uid] != self.dflt_prp
    if (is_bot_prp and prp_other_than_dflt):
      return True
    # (∃pk : ((prp[k] =< 0, s >) ∧ (s != ⊥))
    zero_s_not_bot = [self.prp[k] for k in self.config \
        if self.prp[k] and self.prp[k].tag and self.prp[k].phase == 0]
    # (∃pk, pk' ∈ config : ¬corrDeg(k, k'))
    differing_deg = False
    for k in combinations(self.config, 2):
      if not self.corr_deg(*k):
        differing_deg = True
    # {pk ∈ config : degree(i)+1 mod 6 = degree(k)}
    k_with_diff_deg = [k for k in self.config\
        if self.prp[k] and (self.prp[self.uid].phase+1 % 3 == self.prp[k].phase) and \
            self.prp[self.uid] != self.dflt_prp]
    # (k_with_diff_deg ⊆ allSeenProcessors)
    close_deg_seen = not set(k_with_diff_deg) <= \
        (self.all_seen_processors ) \
        if k_with_diff_deg else False
    if zero_s_not_bot:
      if __debug__:
        print(f"{self.uid} zero_s_not_bot")
      return True
    elif differing_deg:
      if __debug__:
        print("differing_deg")
      return True
    elif close_deg_seen:
      if __debug__:
        print("close_deg_seen")
      return True
    elif (len(self.proposal_set()) > 1):
      if __debug__:
        print("|proposalSet| > 1")
      return True
    else:
      if __debug__:
        print("ok")
      return False

  def no_default_no_bot(self):
    """ Checks that no proposal holds dfltPrp or None (bot).

    Returns:
      bool: True if no proposal is dfltPrp and no proposal is None. False
        otherwise.
    """
    prps = set([self.prp[k] for k in self.config])
    return ( prps != set([self.dflt_prp]) ) and (None not in prps)

  def local_reset(self, tag):
    """ Reset the local environment to only hold the Record with tag tag.

    Args:
      tag (tuple): the record to keep.
    """
    max_tag = self.S.max_phase(['pre', 'fin', 'FIN'])
    if(tag[0] == max_tag[0]):
      self.S.reset(tag)
      self.inc_nbrs = deque(maxlen=self.queue_size)
      self.pre = {}
      self.fin = {}
      self.FIN = {}
