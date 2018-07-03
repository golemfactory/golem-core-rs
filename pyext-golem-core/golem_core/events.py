import inspect
import sys

from abc import ABCMeta
from typing import Tuple

from .enums import TransportProtocol, LogLevel
from .structs import Encapsulated


__all__ = (
    'CoreEvent',
    'BaseEvent',
    'Exiting',
    'Started',
    'Stopped',
    'Connected',
    'Disconnected',
    'Message',
    'Log',
)


class BaseEvent(metaclass=ABCMeta):
    ID = None


class TransportAndAddressEvent(BaseEvent, metaclass=ABCMeta):

    def __init__(self,
                 transport_id: int,
                 address: Tuple[str, int]) -> None:

        self.transport_protocol = TransportProtocol.convert_from(transport_id)
        self.address = address


class Exiting(BaseEvent):
    ID = 0


class Started(TransportAndAddressEvent):
    ID = 1


class Stopped(TransportAndAddressEvent):
    ID = 2


class Connected(TransportAndAddressEvent):
    ID = 100

    def __init__(self,
                 transport_id: int,
                 address: Tuple[str, int],
                 initiator: bool) -> None:

        super().__init__(transport_id, address)
        self.initiator = initiator


class Disconnected(TransportAndAddressEvent):
    ID = 101


class Message(TransportAndAddressEvent):
    ID = 102

    def __init__(self,
                 transport_id: int,
                 address: Tuple[str, int],
                 encapsulated: Encapsulated) -> None:

        super().__init__(transport_id, address)
        self.encapsulated = encapsulated


class Log(BaseEvent):
    ID = 200

    def __init__(
        self,
        log_level: LogLevel,
        message: str,
    ) -> None:

        self.log_level = log_level
        self.message = message


def _collect_events():
    module = sys.modules[__name__]
    es = inspect.getmembers(
        module,
        lambda c: bool(
            inspect.isclass(c) and
            c is not BaseEvent and
            issubclass(c, BaseEvent)
        )
    )
    return {e.ID: e for _, e in es}


class CoreEvent:

    core_events = _collect_events()

    @classmethod
    def convert_from(cls, payload) -> BaseEvent:
        if len(payload) < 1:
            raise ValueError("Event ID is missing")

        event = cls.core_events.get(payload[0])
        return event(*payload[1:])
