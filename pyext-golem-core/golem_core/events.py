import inspect
import sys

from abc import ABCMeta
from typing import Dict, List, Tuple, Type, Union

from .enums import TransportProtocol, LogLevel
from .structs import Encapsulated


__all__ = (
    'Event',
    'Exiting',
    'Started',
    'Stopped',
    'Connected',
    'Disconnected',
    'Message',
    'Log',
)


class Event(metaclass=ABCMeta):

    __events = None

    @classmethod
    def convert_from(cls, payload: Union[List, Tuple]) -> 'Event':
        if len(payload) < 1:
            raise ValueError("Event ID is missing")

        if not cls.__events:
            cls.__events = _subclasses(cls)

        event = cls.__events.get(payload[0])
        return event(*payload[1:])


class TransportAndAddressEvent(Event, metaclass=ABCMeta):

    def __init__(self,
                 transport_id: int,
                 address: Tuple[str, int]) -> None:

        self.transport_protocol = TransportProtocol.convert_from(transport_id)
        self.address = address


class Exiting(Event):
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
                 encapsulated: Tuple[int, bytes]) -> None:

        super().__init__(transport_id, address)
        self.encapsulated = Encapsulated(*encapsulated)


class Log(Event):
    ID = 200

    def __init__(self,
                 log_level: LogLevel,
                 message: str) -> None:

        self.log_level = log_level
        self.message = message


def _subclasses(cls) -> Dict[str, Type]:
    module = sys.modules[__name__]
    events = inspect.getmembers(
        module,
        lambda c: bool(
            inspect.isclass(c) and
            c is not cls and
            issubclass(c, cls)
        )
    )
    return {e.ID: e for _, e in events}
