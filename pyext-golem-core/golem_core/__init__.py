from libgolem_core import *


class Encapsulated:

    def __init__(self, protocol_id: int, message: bytes):
        self.protocol_id = protocol_id
        self.message = message
