from abc import ABCMeta, abstractmethod

import brevitas.nn as qnn
import torch
import torch.nn as nn
import torch.nn.functional as F
from brevitas.inject import ExtendedInjector
from brevitas.quant.experimental.float_base import (
    FloatActBase,
    FloatBase,
    FloatWeightBase,
)
from brevitas.quant.scaled_int import Int8ActPerTensorFloat, Int8WeightPerTensorFloat
from brevitas.quant_tensor.float_quant_tensor import FloatQuantTensor
from brevitas.quant_tensor.int_quant_tensor import IntQuantTensor
from torch import Tensor

__all__ = ["IntQuantLinearModel", "FloatQuantLinearModel", "mf_to_raw", "raw_to_mf"]


class QuantLinearModel(nn.Module, metaclass=ABCMeta):
    # Pass kwargs to QuantLinear layer
    def __init__(self, **kwargs):
        super(QuantLinearModel, self).__init__()
        self.flatten_inp = nn.Flatten()
        self.fc1 = qnn.QuantLinear(
            in_features=28 * 28, out_features=10, bias=False, **kwargs
        )

    def forward(self, x: Tensor) -> Tensor:
        out = self.flatten_inp(x)
        out = self.fc1(out)
        out = F.log_softmax(out, dim=-1)
        return out

    @abstractmethod
    def quant_weight(self) -> tuple[Tensor, Tensor]:
        """
        Get quantized model weights as uints
        """
        pass

    @abstractmethod
    def quant_input(self, x: Tensor) -> tuple[Tensor, Tensor]:
        """
        Quantize tensor as uints
        """
        pass


class IntQuantLinearModel(QuantLinearModel):
    def __init__(self, input_bit_width: int = 8, weight_bit_width: int = 8):
        super(IntQuantLinearModel, self).__init__(
            input_quant=Int8ActPerTensorFloat,
            input_bit_width=input_bit_width,
            weight_quant=Int8WeightPerTensorFloat,
            weight_bit_width=weight_bit_width,
        )

    def quant_weight(self) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            weights: IntQuantTensor = self.fc1.quant_weight()

        return weights.int(), weights.scale

    def quant_input(self, x: Tensor) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            inputs: IntQuantTensor = self.fc1.input_quant(x)

        return inputs.int(), inputs.scale


# Helper for converting quantized floats to raw binary
def mf_to_raw(
    x: Tensor,
    exponent_bit_width: int,
    mantissa_bit_width: int,
    exponent_bias: int,
    eps: float,
) -> Tensor:
    emin = -exponent_bias + 1

    sign = torch.signbit(x).type(torch.int)
    exp = torch.floor(torch.log2(torch.abs(x) + eps)).type(torch.int)
    exp = torch.clamp_min(exp, emin)
    man = torch.abs(x) / torch.exp2(exp)

    exp_bits = exp - (man < 1).type(torch.int) + exponent_bias  # Denorm
    man_bits = (man * (1 << mantissa_bit_width)).type(torch.int)
    man_bits = man_bits & ((1 << mantissa_bit_width) - 1)

    return (
        (sign << (exponent_bit_width + mantissa_bit_width))
        | (exp_bits << mantissa_bit_width)
        | man_bits
    )


# Helper for converting raw binary minifloats to floats
def raw_to_mf(
    x: Tensor,
    exponent_bit_width: int,
    mantissa_bit_width: int,
    exponent_bias: int,
) -> Tensor:
    emin = -exponent_bias + 1

    sign_mask = 1 << (exponent_bit_width + mantissa_bit_width)
    exp_mask = ((1 << exponent_bit_width) - 1) << mantissa_bit_width
    man_mask = (1 << mantissa_bit_width) - 1

    sign = torch.where((x & sign_mask) == 0, 1.0, -1.0)
    exp_denorm = ((x & exp_mask) >> mantissa_bit_width).type(torch.float)
    man_denorm = (x & man_mask).type(torch.float) / (2**mantissa_bit_width)

    exp = torch.where(exp_denorm == 0, emin, exp_denorm - exponent_bias)
    man = torch.where(exp_denorm == 0, man_denorm, 1.0 + man_denorm)

    return sign * man * torch.exp2(exp)


# Helper for creating custom minifloat quantizers
def fp_mixin_factory(
    exponent_bit_width: int, mantissa_bit_width: int, base_class: FloatBase
):
    bit_width = 1 + exponent_bit_width + mantissa_bit_width
    name = f"Fp{bit_width}e{exponent_bit_width}m{mantissa_bit_width}"
    mixin = type(
        name + "Mixin",
        (ExtendedInjector,),
        {
            # Add sign bit
            "bit_width": bit_width,
            "exponent_bit_width": exponent_bit_width,
            "mantissa_bit_width": mantissa_bit_width,
            "saturating": True,
        },
    )

    if base_class is FloatActBase:
        class_name = name + "Act"
    elif base_class is FloatWeightBase:
        class_name = name + "Weight"
    else:
        raise TypeError("Unsupported Float Base")

    return type(class_name, (mixin, base_class), {})


class FloatQuantLinearModel(QuantLinearModel):
    def __init__(
        self,
        input_exponent_bit_width: int = 4,
        input_mantissa_bit_width: int = 3,
        weight_exponent_bit_width: int = 4,
        weight_mantissa_bit_width: int = 3,
    ):
        input_quant = fp_mixin_factory(
            input_exponent_bit_width, input_mantissa_bit_width, FloatActBase
        )
        weight_quant = fp_mixin_factory(
            weight_exponent_bit_width, weight_mantissa_bit_width, FloatWeightBase
        )
        super(FloatQuantLinearModel, self).__init__(
            input_quant=input_quant,
            weight_quant=weight_quant,
        )

    def quant_weight(self) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            weights: FloatQuantTensor = self.fc1.quant_weight()

        raw_weights = mf_to_raw(
            weights.minifloat(),
            weights.exponent_bit_width.int(),
            weights.mantissa_bit_width.int(),
            weights.exponent_bias.int(),
            weights.eps,
        ).type(torch.uint8)

        return raw_weights, weights.scale

    def quant_input(self, x: Tensor) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            inputs: FloatQuantTensor = self.fc1.input_quant(x)

        raw_inputs = mf_to_raw(
            inputs.minifloat(),
            inputs.exponent_bit_width.int(),
            inputs.mantissa_bit_width.int(),
            inputs.exponent_bias.int(),
            inputs.eps,
        ).type(torch.uint8)

        return raw_inputs, inputs.scale
