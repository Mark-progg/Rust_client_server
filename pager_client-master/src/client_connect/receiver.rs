use crate::client_connect::error::Error;
use std::io::Read;
use std::net::TcpStream;

pub struct Receiver { client: TcpStream, }

impl Receiver
{
    pub fn new(client:TcpStream) -> Receiver { Receiver{ client } }
    pub fn get_msg(&mut self) -> (Option<String>, Error) {
        let mut command:[u8;1] = [0];
        if self.client.read(&mut command).is_err() { return (None, Error::NoConnect);}
        match command[0]{
            //  Сообщение
            0 => {
                let mut len:[u8;4] = [0, 0, 0, 0]; //для записи u32 из потока
                self.client.read(&mut len).unwrap();
                let len = u32::from_ne_bytes(len);
                let mut data = Vec::with_capacity(len as usize);
                if self.client.read(&mut data).is_err() { return (None, Error::NoConnect);}
                (Some(String::from_utf8(data).unwrap()), Error::NoError)
            },
            //  Связь с клиентом 2 отсутствует (от сервера к клиенту)
            1 => (None, Error::ClientErr),                                        
            //  Связь со вторым сервером потеряна
            2 => (None, Error::ServerErr),
            _ => { panic!("An unknown command was received from the server!"); },
        }
    }
}