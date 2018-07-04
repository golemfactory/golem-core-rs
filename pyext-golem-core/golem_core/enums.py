from enum import Enum

__all__ = (
    'LogLevel',
    'ErrorKind',
    'TransportProtocol',
)


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


class ErrorKind(Enum):

    Io = 'Io'
    Network = 'Network'
    Mailbox = 'Mailbox'
    Python = 'Python'
    Other = 'Other'

    @staticmethod
    def from_core_error(error):
        error_str = str(error)
        for kind in ErrorKind:
            if error_str.startswith(kind.value):
                return kind
        return ErrorKind.Python


class TransportProtocol(_IntConversionMixin, Enum):

    Tcp = 6
    Udp = 17
    Unsupported = 0
