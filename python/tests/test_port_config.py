from arbolta import PortConfig


def test_port_config():
    x = PortConfig((1, 10), int)

    assert isinstance(x.shape, tuple) and len(x.shape) == 2
    assert x.shape[0] == 1 and x.shape[1] == 10
    assert x.dtype == int
    assert isinstance(x.clock, bool) and isinstance(x.reset, bool)
    assert not x.clock and not x.reset
    assert x.polarity is None
