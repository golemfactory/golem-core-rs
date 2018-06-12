use std::u32::MAX as u32_MAX;

use bincode::{internal, Bounded};
use byteorder::{ByteOrder, BigEndian as BE};
use bytes::{BigEndian, BufMut, BytesMut};
use tokio_io::codec::{Decoder, Encoder};

pub mod error;
pub mod message;

use self::error::CodecError;
use self::message::*;


pub struct MessageCodec;

impl Encoder for MessageCodec {
    type Item = Message;
    type Error = CodecError;

    fn encode(&mut self, msg: Self::Item, bytes: &mut BytesMut)
        -> Result<(), Self::Error>
    {
        let bound = Bounded(u32_MAX as u64);
        let result = internal::serialize::<Message, Bounded, BE>(&msg, bound);
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
        let result = internal::deserialize::<Message, BE>(msg.as_ref());
        if let Err(e) = result {
            return Err(CodecError::from(e));
        }

        Ok(Some(result.unwrap()))
    }
}
