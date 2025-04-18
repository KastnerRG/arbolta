import torch
import torch.nn as nn
from torch import Tensor
from torch.nn.modules.loss import _Loss
from torch.optim.optimizer import Optimizer
from torch.utils.data import DataLoader
from tqdm.notebook import tqdm


def train_for_epoch(
    model: nn.Module,
    device: str,
    train_loader: DataLoader,
    criterion: _Loss,
    optimizer: Optimizer,
) -> None:
    model.train()

    correct_total = 0
    size_total = 0
    with tqdm(train_loader) as tepoch:
        tepoch.set_description("Train")
        for images, targets in tepoch:
            images, targets = images.to(device), targets.to(device)

            optimizer.zero_grad()
            logits: Tensor = model(images)
            loss: Tensor = criterion(logits, targets)
            loss.backward()
            optimizer.step()

            # Compute metrics
            preds = logits.argmax(dim=1, keepdim=True).squeeze()
            num_correct = (preds == targets).sum().item()
            correct_total += num_correct
            size_total += len(targets)
            accuracy = correct_total / size_total

            tepoch.set_postfix(loss=loss.item(), acc=format(accuracy, "3.2%"))


def test_model(model: nn.Module, device: str, test_loader: DataLoader,
               criterion: _Loss) -> None:
    model.eval()

    test_loss = 0
    correct_total = 0
    size_total = 0
    with torch.no_grad():
        with tqdm(test_loader) as tepoch:
            tepoch.set_description("Test")
            for images, targets in tepoch:
                images, targets = images.to(device), targets.to(device)

                logits: Tensor = model(images)
                loss: Tensor = criterion(logits, targets, reduction="sum")

                preds = logits.argmax(dim=1, keepdim=True).squeeze()
                num_correct = (preds == targets).sum().item()
                correct_total += num_correct
                size_total += len(targets)
                accuracy = correct_total / size_total

                test_loss += loss.item()
                avg_loss = test_loss / size_total

                tepoch.set_postfix(
                    loss=loss.item(),
                    acc=format(accuracy, "3.2%"),
                    avg_loss=avg_loss,
                )
