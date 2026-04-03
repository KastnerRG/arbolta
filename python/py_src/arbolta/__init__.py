# Copyright (c) 2026 Alexander Redding
# SPDX-License-Identifier: MIT
from dataclasses import dataclass
from typing import Literal, Optional

import numpy as np
from numpy.typing import DTypeLike

from .arbolta import HardwareDesign, Ports

__all__ = ["HardwareDesign", "Ports", "PortConfig"]


@dataclass
class PortConfig:
    """
    Configuration for a HardwareDesign port.

    :var shape: Interpret port bits with shape, defaults to (1, 1)
    :vartype shape: tuple[int, int]
    :var dtype: Interpret port bits as `dtype`, defaults to `np.uint`
    :vartype dtype: DTypeLike
    :var clock: Port is a clock signal, defaults to False
    :vartype clock: bool
    :var reset: Port is a reset signal, defaults to False
    :vartype reset: bool
    :var polarity: Clock polarity of port, defaults to False
    :vartype polarity: 0 | 1, optional
    """

    shape: tuple[int, int] = (1, 1)
    dtype: DTypeLike = np.uint
    clock: bool = False
    reset: bool = False
    polarity: Optional[Literal[0, 1]] = None
