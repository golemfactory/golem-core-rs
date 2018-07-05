__all__ = (
    'Encapsulated',
)


class Encapsulated:

    def __init__(self, protocol_id: int, message: bytes) -> None:
        self.protocol_id = protocol_id
        self.message = message
