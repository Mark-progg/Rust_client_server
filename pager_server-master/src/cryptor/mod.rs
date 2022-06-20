mod key;
extern crate inotify;
pub use key::Key;
use std::io::{ Read, Write, prelude::*};
use std:: {
    net::TcpListener,
    fs::File,
    time,
    env,
    sync::{ mpsc, mpsc::{Sender, Receiver} },
};
use inotify::{
    EventMask,
    WatchMask,
    Inotify,
};

pub struct Cryptor
{
    key: Key,
    rx : Receiver<Key>,
}

impl Cryptor
{
    pub fn new() -> Result<Self, std::io::Error>
    {
        let (tx, rx): (Sender<Key>, Receiver<Key>) = mpsc::channel();
        let tx_clone= tx.clone();
        let mut key = [0u8;32];
        File::open("key")?.read(&mut key)?;
        let key = Key::new(key);
        std::thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(_) => continue,
                };
                let mut buf = [0u8 ;4096];
                if let Err(err) = stream.read(&mut buf) {
                    println!("Unable to read stream: {}", err);
                }
                let req_str = String::from_utf8_lossy(&buf);
                let req_str = req_str.trim_matches(char::from(0));
                let parts: Vec<&str> = req_str.split("\r\n\r\n").collect();
                match Key::from_vec(parts[1].as_bytes().to_vec())
                {
                    Ok(key) => tx.send(key).unwrap(),
                    Err(err) =>
                    {
                        println!("Curl not get key: {}", err);
                        continue;
                    },
                }
                let response = b"HTTP/1.1 200 OK\r\n";
                if let Err(err) = stream.write(response) {
                    println!("Failed sending response: {}", err);
                }
            }
        });

        let mut inotify = Inotify::init()
            .expect("Failed to initialize inotify");
        let current_dir = env::current_dir()
            .expect("Failed to determine current directory");
        inotify.add_watch(
                current_dir,
                WatchMask::MODIFY | WatchMask::CREATE,
            )
            .expect("Failed to add inotify watch");
        println!("Watching current directory for activity...");
        let mut buffer = [0u8; 4096];

        std::thread::spawn(move || {
           {
               loop {
                   let events = inotify
                       .read_events_blocking(&mut buffer)
                       .expect("Failed to read inotify events");
                   for event in events {
                       if (event.mask.contains(EventMask::MODIFY) ||
                       event.mask.contains(EventMask::CREATE)
                       ) &&
                       !event.mask.contains(EventMask::ISDIR) {
                               std::thread::sleep(time::Duration::from_millis(5));
                               //Code for code to see the key and the modified file
                               let mut key_buffer = [0u8; 32];
                               println!("File modified: {:?}", event.name);
                               let mut f = File::open("X").unwrap();
                               f.read(&mut key_buffer).unwrap();
                               println!("The key_buffer: {:?}", &key_buffer);
                               //send key
                               tx_clone.send(Key::new(key_buffer));
                       }
                   }
               }
           }
        });
        Ok(Cryptor{ key : key, rx })
    }
    fn get_key(&mut self) -> &Key
    {
        loop{
            match self.rx.recv_timeout(time::Duration::from_millis(5))
            {
                Ok(key) => self.key = key,
                Err(_) => break,
            }
        }
        &self.key
    }
    pub fn encryptor(&mut self, msg:String) -> Vec<u8>
    {
        self.get_key().encryptor(msg)
    }

    // Так как шифрование происходит от меньшего байта к большему, то и расшифровывать надо от меньшего бита к большему
    // Пример сборки числа 213 = 0000_0000 - 1000_0000 - 0100_0000 - 1010_0000 - 0101_0000 - 1010_1000 - 0101_0100 - 1010_1010 - 1101_0101
    pub fn decryptor(&mut self, msg:Vec<u8>) -> String
    {
        self.get_key().decryptor(msg)
    }
}
