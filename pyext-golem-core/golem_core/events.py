import inspect
import sys

from abc import ABCMeta
from typing import Tuple

from . import Encapsulated
from .enums import TransportProtocol


class BaseEvent(metaclass=ABCMeta):
    pass


class Exiting(BaseEvent):
    ID = 0


class Started(BaseEvent):
    ID = 1

    def __init__(self, transport_protocol: TransportProtocol) -> None:
        self.transport_protocol = transport_protocol


class Stopped(BaseEvent):
    ID = 2

    def __init__(self, transport_protocol: TransportProtocol) -> None:
        self.transport_protocol = transport_protocol


class Connected(BaseEvent):
    ID = 100

    def __init__(
        self,
        transport_protocol: TransportProtocol,
        address: Tuple[str, int],
    ) -> None:

        self.transport_protocol = transport_protocol
        self.address = address


class Disconnected(BaseEvent):
    ID = 101

    def __init__(
        self,
        transport_protocol: TransportProtocol,
        address: Tuple[str, int],
    ) -> None:

        self.transport_protocol = transport_protocol
        self.address = address


class Message(BaseEvent):
    ID = 102

    def __init__(
        self,
        transport_protocol: TransportProtocol,
        address: Tuple[str, int],
        encapsulated: Encapsulated,
    ) -> None:

        self.transport_protocol = transport_protocol
        self.address = address
        self.encapsulated = encapsulated


class Log(BaseEvent):
    ID = 200

    def __init__(
        self,
        transport_protocol: TransportProtocol,
        address: Tuple[str, int],
        encapsulated: Encapsulated,
    ) -> None:

        self.transport_protocol = transport_protocol
        self.address = address
        self.encapsulated = encapsulated


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
