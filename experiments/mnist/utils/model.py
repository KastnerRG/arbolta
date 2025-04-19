import brevitas.nn as qnn
import torch.nn as nn
import torch.nn.functional as F
from brevitas.quant.scaled_int import (Int8ActPerTensorFloat,
                                       Int8WeightPerTensorFloat)


class QuantModel(nn.Module):

    def __init__(self, input_bit_width: int = 8, weight_bit_width: int = 8):
        super(QuantModel, self).__init__()
        self.flatten_inp = nn.Flatten()
        self.fc1 = qnn.QuantLinear(
            in_features=28 * 28,
            out_features=10,
            bias=False,
            input_quant=Int8ActPerTensorFloat,
            input_bit_width=input_bit_width,
            weight_quant=Int8WeightPerTensorFloat,
            weight_bit_width=weight_bit_width,
        )

    def forward(self, x):
        out = self.flatten_inp(x)
        out = self.fc1(out)
        out = F.log_softmax(out, dim=-1)
        return out
