use crate::client_connect::error::Error;
use std::net::TcpStream;
use std::io::Write;
pub struct Sender
{
    client: TcpStream,
}

impl Sender
{
    pub fn new(client:TcpStream) -> Sender { Sender{ client } }
    pub fn send_msg(&mut self, msg: String) -> Error {
        //Send comand = 0
        if self.client.write(&u8::to_ne_bytes(0)).is_err() { return Error::NoConnect;}
        //Send size
        if self.client.write(&usize::to_ne_bytes(msg.len())).is_err() { return Error::NoConnect;}
        //Send data
        if self.client.write(&msg.as_bytes()).is_err() { return Error::NoConnect;}
        return Error::NoError;
    }
}