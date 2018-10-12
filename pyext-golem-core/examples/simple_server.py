import os
import sys
import time

from typing import Tuple, Optional
from threading import Thread

from golem_core import CoreNetwork, CoreError
from golem_core.enums import *
from golem_core.events import *


class Network:

    def __init__(self, listen_address, connect_address) -> None:
        if not connect_address:
            connect_address = None, None

        self.network = CoreNetwork()
        self.listen_host, self.listen_port = listen_address
        self.connect_host, self.connect_port = connect_address
        self.running = False

        self._is_server = bool(self.connect_host)

    def run(self):
        self.network.run(self.listen_host, self.listen_port)
        self.running = True

        if self.connect_host and self.connect_port:
            self.network.poll(0)
            res = self.network.connect(TransportProtocol.Tcp.value, self.connect_host, self.connect_port)
            print('Connected? {}'.format(res))

        while self.running:
            self._loop()

    def _loop(self):
        try:
            args = self.network.poll(3)
            print('args? {}'.format(args))
        except CoreError:
            return

        if not args:
            return

        try:
            event = Event.convert_from(args)
        except Exception as exc:
            print(f'Invalid args: args: {exc}')
        else:
            self._handle(event)

    def _handle(self, event):
        if isinstance(event, Message):
            msg = f'Message {event}'

            Thread(
                daemon=True,
                target=self.delayed_send,
                args=(
                    TransportProtocol.Tcp.value,
                    ('0.0.0.0', 0),
                    (1 if self._is_server else 2, os.urandom(32)),
                    0.5
                ),
            ).start()
        elif isinstance(event, Exiting):
            msg = 'Exiting'
        elif isinstance(event, Started):
            msg = f'Started {event.transport_protocol}'
        elif isinstance(event, Stopped):
            msg = f'Stopped {event.transport_protocol}'
        elif isinstance(event, Connected):
            msg = f'Connected {event}'
        elif isinstance(event, Disconnected):
            msg = f'Disconnected {event}'
        elif isinstance(event, Log):
            msg = f'Log {event}'
        else:
            msg = 'Unknown event'

        print(msg)

    def delayed_send(self, msg, delay):
        time.sleep(delay)
        self.network.send(*msg)


def address(as_str: str) -> Optional[Tuple[str, int]]:
    split = as_str.split(':')

    try:
        port = int(split[1])
    except (TypeError, ValueError):
        return None

    if port < 0 or port > 65535:
        return None

    return split[0], port


def main(args) -> None:

    if len(args) < 2:
        print('Usage:', args[0])
        print('\t', args[0], 'listen_ip:listen_port [connect_ip:connect_port]')
        sys.exit(1)

    listen_address = address(args[1])
    connect_address = None

    if not listen_address:
        print(f'Invalid address: {args[1]}')
        sys.exit(1)

    if len(args) > 2:
        connect_address = address(args[2])
        if not connect_address:
            print(f'Invalid address: {args[2]}')
            sys.exit(1)

    network = Network(listen_address, connect_address)
    network.run()


if __name__ == '__main__':
    main(sys.argv)
