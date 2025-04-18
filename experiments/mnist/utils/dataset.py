import torchvision
from torch.utils.data import DataLoader


def get_mnist_dataloaders(
    data_root: str,
    num_workers: int = 0,
    batch_size_train: int = 64,
    batch_size_test: int = 1024,
    pin_memory: bool = True,
    download: bool = False,
):
    image_transform = torchvision.transforms.Compose([
        torchvision.transforms.ToTensor(),
        torchvision.transforms.Normalize((0.1307, ), (0.3081, )),
    ])

    train_dataset = torchvision.datasets.MNIST(data_root,
                                               train=True,
                                               download=download,
                                               transform=image_transform)
    test_dataset = torchvision.datasets.MNIST(data_root,
                                              train=False,
                                              download=download,
                                              transform=image_transform)

    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size_train,
        shuffle=True,
        num_workers=num_workers,
        pin_memory=pin_memory,
    )

    test_loader = DataLoader(
        test_dataset,
        batch_size=batch_size_test,
        shuffle=True,
        num_workers=num_workers,
        pin_memory=pin_memory,
    )

    return train_loader, test_loader
