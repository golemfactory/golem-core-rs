from enum import Enum


class _IntConversionMixin:

    @classmethod
    def convert_from(cls, value: int):
        return cls(value)

    def convert_to(self) -> int:
        return int(self.value)


class LogLevel(_IntConversionMixin, Enum):

    Debug = 0
    Info = 1
    Warning = 2
    Error = 3

    def __str__(self):
        return self.name.lower()


class TransportProtocol(_IntConversionMixin, Enum):

    Tcp = 6
    Udp = 17
    Unsupported = 0
