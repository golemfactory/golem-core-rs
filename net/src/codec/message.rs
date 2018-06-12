#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Encapsulated
{
    pub protocol_id: u16,
    pub message: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Message)]
pub enum Message
{
    Encapsulated(Encapsulated),
    Disconnect,
}
