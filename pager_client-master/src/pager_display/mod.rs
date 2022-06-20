use std::io::{Stdin, stdin};
pub struct Display
{
    //Временное наполнение
    outp: Stdin,
}

impl Display 
{
    pub fn new() -> Self
    {
        Display{ outp : stdin() }
    }

    pub fn print(&self, msg : String) -> Result<(), ()>
    {
        println!("{}", msg);
        Ok(())
    }
}