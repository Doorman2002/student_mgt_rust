use serde::Deserialize;

#[derive(Deserialize)]

pub struct Signup{
    pub name:String,
    pub email:String,
    pub phone:String,
    pub course:String,
    pub password:String,
}


#[derive(Deserialize)]

pub struct Login{
    pub email:String,
    pub password:String
}

#[derive(Deserialize)]
pub struct VerifyEmail{
    
    pub otp:String
}


#[derive(Deserialize)]
pub struct Email{
    pub email:String
}

#[derive(Deserialize)]
pub struct ForgottenPassword{
    pub otp:String,
    pub password:String
}
#[derive(Deserialize)]
pub struct EmailInfo{
    pub email:String
}
