#!/usr/bin/env python3
# -------------------------------------------------------------------------------------------------
# <copyright file="analyzers.pxd" company="Invariance Pte">
#  Copyright (C) 2018-2019 Invariance Pte. All rights reserved.
#  The use of this source code is governed by the license as found in the LICENSE.md file.
#  http://www.invariance.com
# </copyright>
# -------------------------------------------------------------------------------------------------

# cython: language_level=3, boundscheck=False, wraparound=False, nonecheck=False

from inv_trader.model.objects cimport Tick


cdef class SpreadAnalyzer:
    """
    Provides a means of analyzing the spread in a market and tracking various
    metrics.
    """
    cdef int _decimal_precision
    cdef int _average_spread_capacity
    cdef list _spreads
    cdef object _average_spreads

    cdef readonly bint initialized
    cdef readonly object average

    cpdef void update(self, Tick tick)
    cpdef void snapshot_average(self)
    cpdef list get_average_spreads(self)
    cpdef void reset(self)

    cdef void _calculate_average(self)
