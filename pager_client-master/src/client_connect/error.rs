#[derive(PartialEq)]
pub enum Error{
    //Connection Error
    NoConnect,
    //Error connecting the second client
    ClientErr,
    //Error connecting the second server
    ServerErr,
    //No errors
    NoError,
}