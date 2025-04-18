import numpy as np
import torch
from arbolta import HardwareDesign


def run_systolic_array(design: HardwareDesign, x: torch.Tensor,
                       k: torch.Tensor) -> torch.Tensor:
    """
    Run inputs through systolic array.
    Expects x: (K,R), k: (K,C) -> y: (C,R)
    """
    K, R, C = x.shape[0], x.shape[1], k.shape[1]
    actual = np.zeros((C, R), dtype=np.int32)

    # Start simulation
    design.eval_reset_clocked()

    for i in range(K):
        while design.ports.s_ready == 0:
            design.eval_clocked()

        design.ports.s_valid = 1
        design.ports.sx_data = x[i]
        design.ports.sk_data = k[i]

        if i == K - 1:
            design.ports.s_last = 1

        design.eval_clocked()

    design.ports.m_ready = 1
    design.ports.s_valid = 0
    design.ports.s_last = 0

    idx = 0
    while True:
        if design.ports.m_valid.item() == 1:
            actual[idx] = design.ports.m_data
            idx += 1

        if design.ports.m_last.item() == 1:
            break

        design.eval_clocked()

    return torch.from_numpy(actual)
