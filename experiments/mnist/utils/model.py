from abc import ABCMeta, abstractmethod

import brevitas.nn as qnn
import torch
import torch.nn as nn
import torch.nn.functional as F
from brevitas.quant.experimental.float_base import (
    FloatActBase,
    FloatWeightBase,
)
from brevitas.quant.scaled_int import Int8ActPerTensorFloat, Int8WeightPerTensorFloat
from brevitas.quant_tensor.float_quant_tensor import FloatQuantTensor
from brevitas.quant_tensor.int_quant_tensor import IntQuantTensor
from torch import Tensor
from .minifloat import mf_to_raw, fp_mixin_factory

__all__ = [
    "IntQuantLinearModel",
    "FloatQuantLinearModel",
]


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

    def quant_weight(self, return_raw: bool = False) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            weights: FloatQuantTensor = self.fc1.quant_weight()

        out = weights.minifloat()

        if return_raw:
            out = mf_to_raw(
                out,
                weights.exponent_bit_width.int(),
                weights.mantissa_bit_width.int(),
                weights.exponent_bias.int(),
                weights.eps,
            ).type(torch.uint8)

        return out, weights.scale

    def quant_input(self, x: Tensor, return_raw: bool = False) -> tuple[Tensor, Tensor]:
        with torch.no_grad():
            inputs: FloatQuantTensor = self.fc1.input_quant(x)

        out = inputs.minifloat()

        if return_raw:
            out = mf_to_raw(
                out,
                inputs.exponent_bit_width.int(),
                inputs.mantissa_bit_width.int(),
                inputs.exponent_bias.int(),
                inputs.eps,
            ).type(torch.uint8)

        return out, inputs.scale
