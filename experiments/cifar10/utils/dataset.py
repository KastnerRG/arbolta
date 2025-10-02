# Based on https://github.com/Xilinx/brevitas/blob/dev/src/brevitas_examples/imagenet_classification/a2q/utils.py
import numpy as np
import torchvision
import torchvision.transforms as transforms
from torch.utils.data import DataLoader, Dataset, Subset


def get_cifar10_dataloaders(
    data_root: str,
    batch_size_train: int = 128,
    batch_size_test: int = 100,
    num_workers: int = 0,
    pin_memory: bool = True,
    pin_memory_device: str = "cpu",
    download: bool = False,
) -> tuple[DataLoader, DataLoader]:
    mean, std = [0.491, 0.482, 0.447], [0.247, 0.243, 0.262]

    # Transformations
    transform_train = transforms.Compose(
        [
            transforms.RandomCrop(32, padding=4),
            transforms.RandomHorizontalFlip(),
            transforms.ToTensor(),
            transforms.Normalize(mean=mean, std=std),
        ]
    )

    transform_test = transforms.Compose(
        [
            transforms.ToTensor(),
            transforms.Normalize(mean=mean, std=std),
        ]
    )

    train_dataset = torchvision.datasets.CIFAR10(
        root=data_root,
        train=True,
        download=download,
        transform=transform_train,
    )

    test_dataset = torchvision.datasets.CIFAR10(
        root=data_root,
        train=False,
        download=download,
        transform=transform_test,
    )

    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size_train,
        shuffle=True,
        num_workers=num_workers,
        pin_memory=pin_memory,
        pin_memory_device=pin_memory_device,
        persistent_workers=num_workers > 0,  # Remove?
    )

    test_loader = DataLoader(
        test_dataset,
        batch_size=batch_size_test,
        shuffle=False,
        num_workers=num_workers,
        pin_memory=pin_memory,
        pin_memory_device=pin_memory_device,
        persistent_workers=num_workers > 0,  # Remove?
    )

    return train_loader, test_loader


def create_calibration_dataloader(
    dataset: Dataset,
    batch_size: int = 256,
    num_workers: int = 0,
    subset_size: int = 1000,
    pin_memory: bool = True,
    pin_memory_device: str = "cpu",
) -> DataLoader:
    all_indices = np.arange(len(dataset))
    cur_indices = np.random.choice(all_indices, size=subset_size)
    subset = Subset(dataset, cur_indices)
    loader = DataLoader(
        subset,
        batch_size=batch_size,
        num_workers=num_workers,
        pin_memory=pin_memory,
        pin_memory_device=pin_memory_device,
    )
    return loader
