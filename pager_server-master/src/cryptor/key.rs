extern crate rand;
extern crate num_cpus;
use rand::seq::SliceRandom;
use plotlib::{
    page::Page,
    repr::Plot,
    view::ContinuousView,
    style::PointStyle
};
use std::time::Instant;
use rayon::prelude::*;

//key 256 bit or 32 byte
#[derive(Clone, PartialEq)]
pub struct Key
{
    //Contains numbers of bits with 1
    key_true : Vec<u8>,
    //Contains numbers of bit with 0
    key_false: Vec<u8>,
}
impl Key{
    //What to do with key 11111... or 00000... ????
    pub fn new(key_data:[u8;32]) -> Self
    {
        let mut key_true:Vec<u8> = Vec::new();
        let mut key_false:Vec<u8> = Vec::new();
        let mut number : u16 = 0;
        for i in key_data {
            let mut k = i;
            for _ in 0..8
            {
                if k&1 == 1 { key_true.push(number as u8); }
                else { key_false.push(number as u8 ); }
                k>>=1;
                number += 1;
            }
        }
        Key{ key_false, key_true }
    }
    pub fn from_vec(key_data:Vec<u8>) -> Result<Self, String>
    {
        if key_data.len() < 32 { return Err(String::from("The key was less than 32 bytes!")); }
        let mut key_res = [0u8;32];
        for i in 0..32 { key_res[i] = *key_data.get(i).unwrap() }
        Ok(Key::new(key_res))
    }
    
    //Шифрование данных происходит от меньшего бита к большему, типа число 213(11010101) будет записано в вектор как (10101011)
    pub fn encryptor(&self, msg:String) -> Vec<u8>{
        let vec_msg:Vec<u8> = msg.into_bytes();
        let mut encrypt_msg:Vec<u8> = vec![0; vec_msg.len()*8];
        vec_msg
        .par_chunks(optimal_chunk_size(&vec_msg.len()))
        .zip(encrypt_msg.par_chunks_mut(optimal_chunk_size(&vec_msg.len())*8))
        .for_each(| ( slice_msg, slice_encrypt ) | {//Теперь в зависимости от количества ядер процессора нужно расчитать длину чанка чтобыформировалось нужное количество потоков
            let mut rng = rand::thread_rng();
            let mut count = 0;
            for i in slice_msg {
                let mut k = *i;
                for _ in 0..8 {
                    if k&1 == 1 {
                        slice_encrypt[count] = *self.key_true.choose(&mut rng).unwrap();
                    }else{
                        slice_encrypt[count] = *self.key_false.choose(&mut rng).unwrap();
                    }
                    k>>=1;
                    count+=1;
                }
            }
        });
        encrypt_msg
    }

    // Так как шифрование происходит от меньшего байта к большему, то и расшифровывать надо от меньшего бита к большему
    // Пример сборки числа 213 = 0000_0000 - 1000_0000 - 0100_0000 - 1010_0000 - 0101_0000 - 1010_1000 - 0101_0100 - 1010_1010 - 1101_0101
    pub fn decryptor(&self, msg:Vec<u8>) -> String{
        if msg.len() % 8 != 0 {
            panic!("Something get wrong with message");
        }
        let mut bytes_vec = vec![0; msg.len()/8];
        msg
        .par_chunks(optimal_chunk_size(&msg.len()))
        .zip(bytes_vec.par_chunks_mut(optimal_chunk_size(&msg.len())/8))
        .for_each(| ( slice, vec ) | {//Теперь в зависимости от количества ядер процессора нужно расчитать длину чанка чтобыформировалось нужное количество потоков
            let mut count = 0;
            for item in vec {
                //let mut k:u8 = 0;
                for i in 8*count..8*(count + 1) {//Slice равен 16, а мне нужно каждые 8 чекать
                    *item >>= 1;
                    if self.key_true.contains(&slice[i]) {
                        *item += 128;
                    }   
                }
                count += 1;
            }
        });
        String::from_utf8_lossy(&bytes_vec).to_string()
    }
}

fn optimal_chunk_size(message_size: &usize) -> usize {
    if *message_size <= 128 { return 8; }
    let chunk_size = message_size / (num_cpus::get());
    chunk_size - chunk_size % 8
}

// В тестах не отработаны ситуации:
// С ключами из одних 0000... или 1111... потому что мы не знаем, что с этим делать
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encryptor_and_decryptor_works1() {
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        //Цикл сделан из-за рандомайзера в шифраторе
        for _ in 0..1000 {
            assert_eq!("Some message", key.decryptor(key.encryptor(String::from("Some message"))));
        }
    }

    #[test]
    fn encryptor_and_decryptor_works2() {
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        for _ in 0..1000 {
            assert_eq!("Some message\r\n", key.decryptor(key.encryptor(String::from("Some message\r\n"))));
        }
    }

    #[test]
    fn encryptor_and_decryptor_works3() {
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        for _ in 0..1000 {
            assert_eq!("\r\n", key.decryptor(key.encryptor(String::from("\r\n"))));
        }
    }

    #[test]
    fn encryptor_and_decryptor_works4() {
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        for _ in 0..1000 {
            assert_eq!("About
            About us
            Bitcoin Core is an open source project which maintains and releases Bitcoin client software called “Bitcoin Core”.
            
            It is a direct descendant of the original Bitcoin software client released by Satoshi Nakamoto after he published the famous Bitcoin whitepaper.
            
            Bitcoin Core consists of both “full-node” software for fully validating the blockchain as well as a bitcoin wallet. The project also currently maintains related software such as the cryptography library libsecp256k1 and others located at GitHub.
            
            Anyone can contribute to Bitcoin Core.
            
            Team
            The Bitcoin Core project has a large open source developer community with many casual contributors to the codebase. There are many more who contribute research, peer review, testing, documentation, and translation.
            
            Maintainers
            Project maintainers have commit access and are responsible for merging patches from contributors. They perform a janitorial role merging patches that the team agrees should be merged. They also act as a final check to ensure that patches are safe and in line with the project goals. The maintainers’ role is by agreement of project contributors.
            
            Contributors
            Everyone is free to propose code changes and to test, review and comment on open Pull Requests. Anyone who contributes code, review, test, translation or documentation to the Bitcoin Core project is considered a contributor. The release notes for each Bitcoin Core software release contain a credits section to recognize all those who have contributed to the project over the previous release cycle. A list of code contributors can be found on Github.
            
            ", key.decryptor(key.encryptor(String::from("About
            About us
            Bitcoin Core is an open source project which maintains and releases Bitcoin client software called “Bitcoin Core”.
            
            It is a direct descendant of the original Bitcoin software client released by Satoshi Nakamoto after he published the famous Bitcoin whitepaper.
            
            Bitcoin Core consists of both “full-node” software for fully validating the blockchain as well as a bitcoin wallet. The project also currently maintains related software such as the cryptography library libsecp256k1 and others located at GitHub.
            
            Anyone can contribute to Bitcoin Core.
            
            Team
            The Bitcoin Core project has a large open source developer community with many casual contributors to the codebase. There are many more who contribute research, peer review, testing, documentation, and translation.
            
            Maintainers
            Project maintainers have commit access and are responsible for merging patches from contributors. They perform a janitorial role merging patches that the team agrees should be merged. They also act as a final check to ensure that patches are safe and in line with the project goals. The maintainers’ role is by agreement of project contributors.
            
            Contributors
            Everyone is free to propose code changes and to test, review and comment on open Pull Requests. Anyone who contributes code, review, test, translation or documentation to the Bitcoin Core project is considered a contributor. The release notes for each Bitcoin Core software release contain a credits section to recognize all those who have contributed to the project over the previous release cycle. A list of code contributors can be found on Github.
            
            "))));
        }
    }

    #[test]
    fn encryptor_and_decryptor_works5() {
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        for _ in 0..1000 {
            assert_eq!("", key.decryptor(key.encryptor(String::from(""))));
        }
    }

    #[test]
    #[ignore]
    fn encryptor_and_decryptor_speed_test_bytes_in_msg_170()
    {
        let mut data = Vec::new();
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        //Цикл сделан из-за рандомайзера в шифраторе
        let mut msg = String::new();
        for i in 0..10000 { 
            let start = Instant::now();
            msg.push(170 as char);
            assert_eq!(msg, key.decryptor(key.encryptor(String::from(msg.clone()))));
            let duration = start.elapsed();
            data.push((i as f64, duration.as_secs_f64()));
        }
        let s: Plot = Plot::new(data).point_style(
            PointStyle::new() // Стандартный маркер
                .colour("#35C788"),
        );
        // The 'view' describes what set of data is drawn
        let v = ContinuousView::new()
            .add(s)
            .x_label("Some varying variable")
            .y_label("The response of something");
        // A page with a single view is then saved to an SVG file
        Page::single(&v).save("encryptor_and_decryptor_speed_test_bytes_in_msg_170.svg").unwrap();
    }
    
    #[test]
    #[ignore]
    fn encryptor_and_decryptor_speed_test_bytes_in_msg_255()
    {
        let mut data = Vec::new();
        let key = Key::new([8, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48, 148, 56, 156, 245 , 6, 56, 75, 167, 190, 200, 23, 1, 56, 255, 245, 48]);
        //Цикл сделан из-за рандомайзера в шифраторе
        let mut msg = String::new();
        for i in 0..10000 { 
            let start = Instant::now();
            msg.push(255 as char);
            assert_eq!(msg, key.decryptor(key.encryptor(String::from(msg.clone()))));
            let duration = start.elapsed();
            data.push((i as f64, duration.as_secs_f64()));
        }
        let s: Plot = Plot::new(data).point_style(
            PointStyle::new() // Стандартный маркер
                .colour("#35C788"),
        );
        // The 'view' describes what set of data is drawn
        let v = ContinuousView::new()
            .add(s)
            .x_label("Some varying variable")
            .y_label("The response of something");
        // A page with a single view is then saved to an SVG file
        Page::single(&v).save("encryptor_and_decryptor_speed_test_bytes_in_msg_255.svg").unwrap();
    }
}