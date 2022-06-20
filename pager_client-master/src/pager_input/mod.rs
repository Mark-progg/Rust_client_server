use std::io::Stdin;
use std::io::stdin;
pub struct InputDevice
{
    inp: Stdin,
}

impl InputDevice
{
    pub fn new() -> Self {
        InputDevice{inp : stdin() }
    }

    pub fn read(&self) -> String
    {
        let mut msg = String::new();
        //TODO get msg from other input device;
        self.inp.read_line(&mut msg).unwrap();
        msg
    }
}