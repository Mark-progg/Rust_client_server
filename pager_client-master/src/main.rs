mod client_connect;
mod pager_input;
use client_connect::error::Error;
mod pager_display;
use std:: {
    time,
    sync::{ mpsc, mpsc::{Sender, Receiver} },
};
fn main() {
    //все обернуто в беск-ый цикл
    loop
    {
        //коннектимся в серверу , все проверки в ф-ии new() в client_connect::ClinetConnect
        let (mut err, connect) =  client_connect::ClinetConnect::new("127.0.0.1:5555");
        //проверяем на отсутствие соединения
        if err == Error::NoConnect { 
            println!("Server not availible!"); 
            continue;
        }
        //проверяем , что пришло "что-то"
        let mut connect = match connect{
            Some(val) => val,
            None => continue,
        };

        loop
        {
            //еще одна проверка, если не доступен 2-ой клиент или сервер пытаемся
            //подключиться раз в две секунды БЕСКОНЕЧНО надо подумать над этим
            match connect.reconnect()
            {
                Error::ClientErr => { println!("Second clinet not connected!"); } ,
                Error::NoConnect => { err = Error::NoConnect; break; },
                Error::NoError => { err = Error::NoError; break; },
                Error::ServerErr => { println!("Second server not connected!"); },
            }
            std::thread::sleep(time::Duration::from_secs(2));
        } 
        if err == Error::NoConnect { 
            println!("Server not availible!"); 
            continue;
        }
        println!("First server connected!");
        println!("Second server connected!");
        println!("Second client connected!");
        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();
        let mut recv = connect.get_recv();
        let mut send = connect.get_sendr();
        let sender = std::thread::spawn(move || {
            let input_dev = pager_input::InputDevice::new();
            loop
            {   
                //msg это введенная строка из потока на устройстве пользователя, read() привести её к string
                let msg = input_dev.read();
                if rx.recv_timeout(time::Duration::from_millis(1)).is_ok() { break; }
                if send.send_msg(msg) == Error::NoConnect { break; };
            }
        });
        let receiver = std::thread::spawn(move || {
            let disp = pager_display::Display::new();
            loop{
                let (msg, err) = recv.get_msg();
                match err
                {
                    Error::ClientErr => { println!("Second clinet not connected!"); break; } ,
                    Error::NoConnect => { println!("First server not connected!"); break; },
                    Error::NoError => { 
                        let msg = match msg{
                            Some(val) => val,
                            None => panic!("Msg was None!!!"),
                        };
                        disp.print(msg).unwrap();
                    },
                    Error::ServerErr => { println!("Second server not connected!");  break; },
                }
            }
        });
        receiver.join().unwrap();
        tx.send(()).unwrap();
        println!("Press Enter to reconnect!");
        sender.join().unwrap();
    }
}
