
pub mod error;
pub mod receiver;
pub mod sender;
use crate::client_connect::error::Error;
use std::{ io::Read, net::TcpStream };
pub struct ClinetConnect
{
    client: TcpStream,
}
/*
0 Сообщение
1 Связь с клиентом 2 отсутствует (от сервера к клиенту)
2 Связь со вторым сервером потеряна
3 Канал доступен
*/
fn check_command(client: &mut TcpStream) -> Error
{
    let mut command:[u8;1] = [0];
    //считали команду в байт и проверили на ошибку сразу
    if client.read(&mut command).is_err() { 
        return Error::NoConnect; 
    }
    //распарсили ошибку
    match command[0] {
        //Сообщение было получено с сервера, а ожидалась команда 
        0 => { panic!("A message was received from the server, but the command was expected!"); }, 
        1 => { return Error::ClientErr; }, 
        2 => { return Error::ServerErr; }, 
        3 => { return Error::NoError; },
        _ => { panic!("An unknown command was received from the server!"); },
    }
}
impl ClinetConnect{
    pub fn new(ip_adress :&str) -> (Error, Option<ClinetConnect>) {
        match TcpStream::connect(ip_adress)
        {
            Ok(mut client) => {
                return (check_command(&mut client), Some(ClinetConnect { client }));
            }
            Err(_var) => { return (Error::NoConnect, None); }
        }
    }
    pub fn reconnect(&mut self) -> Error {
        return check_command(&mut self.client);
    }
    pub fn get_recv(&mut self) -> receiver::Receiver
    {
        return receiver::Receiver::new (self.client.try_clone().expect("Recv cannot be create!"));
    }
    pub fn get_sendr(&mut self) -> sender::Sender
    {
        return sender::Sender::new (self.client.try_clone().expect("Sender cannot be create!"));
    }

}