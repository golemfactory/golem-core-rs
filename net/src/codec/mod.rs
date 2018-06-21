use std::u32::MAX as u32_MAX;

use bincode::Bounded;
use bincode::internal::{serialize, deserialize};
use byteorder::{ByteOrder, BigEndian as Order};
use bytes::{BigEndian, BufMut, BytesMut};
use tokio_codec::{Decoder, Encoder};

pub mod error;
pub mod message;

use self::error::CodecError;
use self::message::Message;


pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = CodecError;

    fn encode(&mut self, value: Self::Item, bytes: &mut BytesMut)
        -> Result<(), Self::Error>
    {
        let size_limit = Bounded(u32_MAX as u64);
        let result = serialize::<Message, Bounded, Order>(&value, size_limit);
        let ser = match result {
            Ok(ser) => ser,
            Err(e) => {
                return Err(CodecError::from(e));
            }
        };

        let ser_ref: &[u8] = ser.as_ref();
        bytes.reserve(ser_ref.len() + 4);
        bytes.put_u32_be(ser_ref.len() as u32);
        bytes.put(ser_ref);

        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, bytes: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error>
    {
        if bytes.len() < 4 {
            return Ok(None);
        }

        let size = BigEndian::read_u32(bytes.as_ref()) as usize;

        bytes.split_to(4);
        if bytes.len() < size {
            return Ok(None);
        }

        let msg = bytes.split_to(size);
        match deserialize::<Message, Order>(msg.as_ref()) {
            Ok(m) => Ok(Some(m)),
            Err(e) => Err(CodecError::from(e))
        }
    }
}
