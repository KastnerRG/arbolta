# Copyright (c) 2026 Alexander Redding
# SPDX-License-Identifier: MIT

from os import PathLike
from typing import Any, Optional, overload

import numpy as np
from networkx import DiGraph
from numpy.typing import ArrayLike

from . import Bit, CellMapping, PortConfig

class Signal:
    """
    Access to design net
    """

    net: int
    value: Bit
    constant: bool
    toggles_rising: int
    toggles_falling: int

class Signals:
    """
    Access to design signals
    """
    @overload
    def __getitem__(self, net: int) -> Signal: ...
    @overload
    def __getitem__(self, net_name: str) -> list[Signal]: ...
    def __len__(self) -> int: ...

# TODO: Add `raises` docs
class Ports:
    """
    Access to simulated module ports. Port names accessed as attributes.
    """
    def __getattr__(self, name: str) -> np.ndarray: ...
    def __setattr__(self, name: str, value: np.ndarray | ArrayLike) -> None: ...

    """
    Parameters
    ----------
    netlist_path : str | Path | PathLike
        Path to Yosys netlist JSON.
    netlist_path : str | Path | PathLike
        Path to Yosys topological order.
    config : dict[str, PortConfig]
        Configuration for design ports.
    top_module : str, optional
        Name of top module.
    """

class HardwareDesign:
    """
    Simulated hardware design

    :param config: Configuration for design ports
    :type config: dict[str, PortConfig]
    :param netlist_path: Path to Yosys netlist JSON
    :type netlist_path: str | Path | PathLike, optional
    :param torder_path: Path to Yosys topological order
    :type torder_path: str | Path | PathLike, optional
    :param hierarchy_separator: Additional hierarchy separator for submodules
    :type hierarchy_separator: str, optional
    :param top_module: Name of top module, defaults to None (find automatically)
    :type top_module: str, optional
    :param cell_mapping: Define additional cell types
    :type cell_mapping: dict[str, tuple[str, Optional[dict[str, str]]]], optional
    :param design: Serialized design
    :type design: bytes, optional

    :var top_module: Top module of design
    :vartype top_module: str
    :var ports: Access to simulated module ports
    :vartype ports: Ports
    :var modules: List of all submodules in design
    :vartype modules: list[str]
    """

    top_module: str
    ports: Ports
    signals: Signals
    modules: list[str]
    config: dict[str, PortConfig]

    def __init__(
        self,
        config: dict[str, PortConfig],
        netlist: Optional[str | PathLike[str] | bytes] = None,
        torder: Optional[str | PathLike[str] | bytes] = None,
        hierarchy_separator: Optional[str] = None,
        top_module: Optional[str] = None,
        cell_mapping: Optional[CellMapping] = None,
        design: Optional[bytes] = None,
    ): ...
    def __getnewargs_ex__(self) -> tuple[tuple, dict]: ...
    def __getstate__(self) -> Any: ...
    def reset(self) -> None:
        """
        Reset all design signals and registers to zero.
        Resets all toggle to zero.
        """

    def eval(self) -> None:
        """
        Evaluates all cells in design.
        """

    def eval_clocked(self, cycles: Optional[int] = None) -> None:
        """
        Clocks and evaluates design for `cycles`.

        :param cycles: Number of cycles to clock design, defaults to 1
        :type cycles: int, optional

        :raises AttributeError: No clock signal configured
        """

    def eval_reset_clocked(self, cycles: Optional[int] = None) -> None:
        """
        Asserts reset signal and clocks design for `cycles`.

        :param cycles: Number of cycles to clock design, defaults to 1
        :type cycles: int, optional

        :raises AttributeError: No clock signal configured
        :raises AttributeError: No reset signal configured
        """

    def set_signal(self, net: int, val: Bit) -> None: ...
    def get_signal(self, net: int) -> Bit: ...
    def toggle_signal(self, net: int) -> None: ...
    def stick_signal(self, net: int, val: Bit) -> None: ...
    def unstick_signal(self, net: int) -> None: ...
    def toggle_count(
        self, category: str = "total", by_net: bool = True
    ) -> dict[str, dict[str, int]] | dict[str, int]: ...
    def netlist(self) -> dict: ...
    def netlist_graph(self) -> DiGraph: ...
    def submodule_nets(self) -> dict[str, dict[str, list[int]]]: ...
    def submodule_net_values(self) -> dict[str, dict[str, list[bool]]]: ...
