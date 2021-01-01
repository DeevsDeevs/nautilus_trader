# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2021 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from cpython.datetime cimport datetime
from cpython.datetime cimport timedelta

from nautilus_trader.model.c_enums.order_side cimport OrderSide
from nautilus_trader.model.c_enums.position_side cimport PositionSide
from nautilus_trader.model.currency cimport Currency
from nautilus_trader.model.events cimport OrderFilled
from nautilus_trader.model.identifiers cimport AccountId
from nautilus_trader.model.identifiers cimport ClientOrderId
from nautilus_trader.model.identifiers cimport ExecutionId
from nautilus_trader.model.identifiers cimport PositionId
from nautilus_trader.model.identifiers cimport StrategyId
from nautilus_trader.model.identifiers cimport Symbol
from nautilus_trader.model.objects cimport Money
from nautilus_trader.model.objects cimport Price
from nautilus_trader.model.objects cimport Quantity


cdef class Position:
    cdef list _events
    cdef object _buy_quantity
    cdef object _sell_quantity
    cdef dict _commissions

    cdef readonly PositionId id
    """The positions identifier.This may be generated by the exchange/brokerage, or can be system
        generated depending on `Order Management System (OMS)` settings.\n\n\n:returns: `PositionId`"""
    cdef readonly AccountId account_id
    """The account identifier associated with the position.\n\n:returns: `AccountId`"""
    cdef readonly ClientOrderId from_order
    """The client order identifier for the order which first opened the position.\n\n:returns: `ClientOrderId`"""
    cdef readonly StrategyId strategy_id
    """The strategy identifier associated with the position.\n\n:returns: `StrategyId`"""
    cdef readonly Symbol symbol
    """The positions symbol.\n\n:returns: `Symbol`"""
    cdef readonly OrderSide entry
    """The entry direction from open.\n\n:returns: `OrderSide`"""
    cdef readonly PositionSide side
    """The current position side.\n\n:returns: `PositionSide`"""
    cdef readonly object relative_quantity
    """The relative quantity (positive for LONG, negative for SHORT).\n\n:returns: `Decimal`"""
    cdef readonly Quantity quantity
    """The current open quantity.\n\n:returns: `Quantity`"""
    cdef readonly Quantity peak_quantity
    """The peak directional quantity reached by the position.\n\n:returns: `Quantity`"""
    cdef readonly Currency quote_currency
    """The positions quote currency.\n\n:returns: `Currency`"""
    cdef readonly bint is_inverse
    """If the quantity is expressed in quote currency.\n\n:returns: `bool`"""
    cdef readonly datetime timestamp
    """The positions initialization timestamp.\n\n:returns: `datetime`"""
    cdef readonly datetime opened_time
    """The opened time.\n\n:returns: `datetime`"""
    cdef readonly datetime closed_time
    """The closed time.\n\n:returns: `datetime` or `None`"""
    cdef readonly timedelta open_duration
    """The total open duration.\n\n:returns: `timedelta` or `None`"""
    cdef readonly object avg_open
    """The average open price.\n\n:returns: `Decimal`"""
    cdef readonly object avg_close
    """The average closing price.\n\n:returns: `Decimal` or `None`"""
    cdef readonly object realized_points
    """The realized points of the position.\n\n:returns: `Decimal`"""
    cdef readonly object realized_return
    """The realized return of the position.\n\n:returns: `Decimal`"""
    cdef readonly Money realized_pnl
    """The realized P&L of the position (including commission).\n\n:returns: `Money`"""
    cdef readonly Money commission
    """The commission generated by the position in quote currency.\n\n:returns: `Money`"""

    cdef list cl_ord_ids_c(self)
    cdef list order_ids_c(self)
    cdef list execution_ids_c(self)
    cdef list events_c(self)
    cdef OrderFilled last_event_c(self)
    cdef ExecutionId last_execution_id_c(self)
    cdef int event_count_c(self) except *
    cdef str status_string_c(self)
    cdef bint is_long_c(self) except *
    cdef bint is_short_c(self) except *
    cdef bint is_open_c(self) except *
    cdef bint is_closed_c(self) except *

    @staticmethod
    cdef inline PositionSide side_from_order_side_c(OrderSide side) except *

    cpdef void apply(self, OrderFilled event) except *

    cpdef Money notional_value(self, Price last)
    cpdef Money calculate_pnl(self, avg_open, avg_close, quantity)
    cpdef Money unrealized_pnl(self, Price last)
    cpdef Money total_pnl(self, Price last)
    cpdef list commissions(self)

    cdef inline void _handle_buy_order_fill(self, OrderFilled event) except *
    cdef inline void _handle_sell_order_fill(self, OrderFilled event) except *
    cdef inline object _calculate_avg_price(self, avg_price, quantity, OrderFilled event)
    cdef inline object _calculate_avg_open_price(self, OrderFilled event)
    cdef inline object _calculate_avg_close_price(self, OrderFilled event)
    cdef inline object _calculate_points(self, avg_open, avg_close)
    cdef inline object _calculate_points_inverse(self, avg_open, avg_close)
    cdef inline object _calculate_return(self, avg_open, avg_close)
    cdef inline object _calculate_pnl(self, avg_open, avg_close, quantity)
