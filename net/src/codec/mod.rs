use std::mem;

use bincode::internal::{deserialize, serialize};
use bincode::Bounded;
use byteorder::{BigEndian as Order, ByteOrder};
use bytes::{BigEndian, BufMut, BytesMut};
use tokio_codec::{Decoder, Encoder};

pub mod error;
pub mod message;

use self::error::CodecError;
use self::message::Message;

type LenType = u32;
const LEN_SZ: usize = mem::size_of::<LenType>();
const LEN_MAX: LenType = LenType::max_value();


pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = CodecError;

    fn encode(&mut self, value: Self::Item, bytes: &mut BytesMut) -> Result<(), Self::Error> {
        let size_limit = Bounded(LEN_MAX as u64);
        let result = serialize::<Message, Bounded, Order>(&value, size_limit);
        let ser = match result {
            Ok(ser) => ser,
            Err(e) => {
                return Err(CodecError::from(e));
            }
        };

        let ser_ref: &[u8] = ser.as_ref();
        bytes.reserve(ser_ref.len() + LEN_SZ);
        bytes.put_u32_be(ser_ref.len() as LenType);
        bytes.put(ser_ref);

        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = CodecError;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if bytes.len() < LEN_SZ {
            return Ok(None);
        }

        let size = BigEndian::read_u32(bytes.as_ref()) as usize;
        if bytes.len() < size + LEN_SZ {
            return Ok(None);
        }

        bytes.split_to(LEN_SZ);

        let msg = bytes.split_to(size);
        match deserialize::<Message, Order>(msg.as_ref()) {
            Ok(m) => Ok(Some(m)),
            Err(e) => Err(CodecError::from(e)),
        }
    }
}
