#[derive(Debug)]
#[derive(PartialEq,Eq,Hash)]
pub enum Environment{
    Local,
    Staging,
    Preproduction,
    Production
}