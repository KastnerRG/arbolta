from dataclasses import dataclass
from os import PathLike
from pathlib import Path
from typing import Literal, Optional

import numpy as np
from networkx import DiGraph
from numpy.typing import ArrayLike, DTypeLike

@dataclass
class PortConfig:
    """
    Configuration for a HardwareDesign port.

    :var shape: Interpret port bits with shape, defaults to (1, 1)
    :vartype shape: tuple[int, int]
    :var dtype: Interpret port bits as `dtype`, defaults to `np.uint`
    :vartype dtype: DTypeLike, optional
    :var clock: Port is a clock signal, defaults to None
    :vartype clock: bool, optional
    :var reset: Port is a reset signal, defaults to None
    :vartype reset: bool, optional
    :var polarity: Clock polarity of port, defaults to None
    :vartype polarity: 0 | 1, optional
    """

    shape: tuple[int, int] = (1, 1)
    dtype: Optional[DTypeLike] = np.uint
    clock: Optional[bool] = None
    reset: Optional[bool] = None
    polarity: Optional[Literal[0, 1]] = None

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

    :param netlist_path: Path to Yosys netlist JSON
    :type netlist_path: str | Path | PathLike
    :param torder_path: Path to Yosys topological order
    :type torder_path: str | Path | PathLike
    :param config: Configuration for design ports
    :type config: dict[str, PortConfig]
    :param hierarchy_separator: Additional hierarchy separator for submodules
    :type hierarchy_separator: str, optional
    :param top_module: Name of top module, defaults to None (find automatically)
    :type top_module: str, optional
    :param cell_mapping: Define additional cell types
    :type cell_mapping: dict[str, tuple[str, Optional[dict[str, str]]]], optional

    :var ports: Access to simulated module ports
    :vartype ports: Ports
    :var modules: List of all submodules in design
    :vartype modules: list[str]
    """

    ports: Ports
    modules: list[str]

    def __init__(
        self,
        netlist_path: str | Path | PathLike,
        torder_path: str | Path | PathLike,
        config: dict[str, PortConfig],
        hierarchy_separator: Optional[str] = None,
        top_module: Optional[str] = None,
        cell_mapping: Optional[dict[str, tuple[str, Optional[dict[str, str]]]]] = None,
    ) -> None: ...
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

    def stick_signal(self, net: int, val: Literal[0, 1]) -> None: ...
    def unstick_signal(self, net: int) -> None: ...
    def toggle_count(
        self, category: str = "total", by_net: bool = True
    ) -> dict[str, dict[str, int]] | dict[str, int]: ...
    def netlist(self) -> dict: ...
    def netlist_graph(self) -> DiGraph: ...
