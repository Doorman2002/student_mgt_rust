

use crate::models::{Email, EmailInfo, ForgottenPassword, Login, Signup, VerifyEmail};
use crate::Credential;
use actix_web::cookie::Cookie;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use chrono;
use lettre::transport::Transport;
use lettre::{Message, SmtpTransport};
use rand::{RngExt};
use sqlx::Row;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn student_signup(
    form: web::Json<Signup>,
    pool: web::Data<PgPool>,
    _cred: web::Data<Credential>,
    mailer: web::Data<SmtpTransport>,
    mail_from: web::Data<String>,
) -> impl Responder {
    println!("The Backend for the student ");
    let name = form.name.clone();
    let email = form.email.clone();
    let phone = form.phone.clone();
    let course = form.course.clone();
    let password = form.password.clone();

    // Check if email already exists
    let email_check =
        sqlx::query!("SELECT email FROM student WHERE email = $1", email)
            .fetch_optional(pool.get_ref())
            .await;

    match email_check {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "info": "Email already in use, please login"
            }));
        }
        Ok(None) => {}
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "info": "Database error checking email",
                "reason": format!("{}", err)
            }));
        }
    }

    // Check if phone already exists
    let phone_check =
        sqlx::query!("SELECT phone FROM student WHERE phone = $1", phone)
            .fetch_optional(pool.get_ref())
            .await;

    match phone_check {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "info": "Phone number already in use, please login"
            }));
        }
        Ok(None) => {}
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "info": "Database error checking phone",
                "reason": format!("{}", err)
            }));
        }
    }

    // Hash the password
    let hashed_password = match hash(&password, DEFAULT_COST) {
        Ok(val) => val,
        Err(err) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "info": "Unable to encrypt the password",
                "reason": format!("{}", err)
            }));
        }
    };

    let otp: i32 = rand::rng().random_range(100000..999999);
    let time_of_data = chrono::Utc::now();

    // Insert into DB
    let insert_result = sqlx::query(
        "INSERT INTO student (name, email, phone, course, password, email_verify, email_otp_createdAT) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING email, email_verify",
    )
    .bind(name)
    .bind(&email)
    .bind(phone)
    .bind(course)
    .bind(hashed_password)
    .bind(otp.to_string())
    .bind(time_of_data)
    .fetch_one(pool.get_ref())
    .await;

    match insert_result {
        Ok(val) => {
            let msg = Message::builder()
                .from(mail_from.as_str().parse().unwrap())
                .to(val.get::<String, _>("email").parse().unwrap())
                .subject("Verify Your Email Address")
                .body(format!("Your OTP is: {}", otp))
                .unwrap();
            let value=msg.clone();
            let mail_cxt = mailer.get_ref().clone();
            let send_mail = web::block(move || mail_cxt.send(&value)).await;
            // print!("{:?}",msg.clone());
            match send_mail {
                Ok(_) => {
                    return     HttpResponse::Ok().json(serde_json::json!({
                "info": "Signup successful. Please verify your email.",
                "email":&email
            }))
                }
                Err(err) => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "info": format!("{}", err)
                    }));
                }
            }
        
        }
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "info": "Failed to save student",
            "reason": format!("{}", err)
        })),
    }
}

pub async fn verify_email(
    form: web::Json<VerifyEmail>,
    pool: web::Data<PgPool>,
    mail: web::Query<Email>,
) -> impl Responder {
    let email = &mail.email;
    let otp = form.otp.clone();
    println!("{}",email);
    let result = sqlx::query!(
        "SELECT email_verify, email_otp_createdAT FROM student WHERE email = $1",
        email
    )
    
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let stored_otp: String = row.email_verify.unwrap();
            // let otp_time=row.get("email_otp_createdAt")
            let created_at: Option<chrono::DateTime<chrono::Utc>> =
                row.email_otp_createdat;

            if stored_otp != otp {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "info": "Invalid OTP"
                }));
            }

            if let Some(created) = created_at {
                let now = chrono::Utc::now();
                let duration = now - created;
                if duration.num_minutes() > 3 {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "info": "OTP has expired"
                    }));
                }
            }

            let update = sqlx::query(
                "UPDATE student SET verified = TRUE, email_verify = 'nil' WHERE email = $1",
            )
            .bind(email)
            .execute(pool.get_ref())
            .await;

            match update {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "info": "Email verified, you can now login"
                })),
                Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "info": format!("{}", err)
                })),
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "info": "Student not found"
        })),
        Err(err) => HttpResponse::InternalServerError().json(serde_json::json!({
            "info": format!("{}", err)
        })),
    }
}

pub async fn login(
    pool: web::Data<PgPool>,
    form: web::Json<Login>,
) -> impl Responder {
    let info = form.into_inner();
    let email = info.email.clone();
    let password = info.password.clone();

    if email.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "info": "Please complete the credentials"
        }));
    }
    let verify=sqlx::query!(
        "SELECT verified FROM student WHERE email=$1",
        email
    ).fetch_one(pool.get_ref())
    .await;
match verify{
    Ok(val)=>{
        if !val.verified.unwrap(){
            return HttpResponse::Locked().json(
                serde_json::json!({
                    "info":"Please verify Your email"
                })
            );
        }
        else{
                let result = sqlx::query_as!(
        Login,
        "SELECT email, password FROM student WHERE email = $1",
        email
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(val) => {
            if let Some(value) = val {
                if value.email != email {
                    return HttpResponse::NotFound().json(serde_json::json!({
                        "info": "Email not registered"
                    }));
                }

                match bcrypt::verify(&password, &value.password) {
                    Ok(matched) => {
                        if !matched {
                            return HttpResponse::BadRequest().json(serde_json::json!({
                                "info": "Password is wrong"
                            }));
                        }

                        let token = Uuid::new_v4();
                        let tk = sqlx::query(
                            "UPDATE student SET token_id = $1 WHERE email = $2",
                        )
                        .bind(token)
                        .bind(&email)
                        .execute(pool.get_ref())
                        .await;

                        match tk {
                            Ok(_) => {}
                            Err(err) => {
                                return HttpResponse::BadRequest().json(serde_json::json!({
                                    "info": format!("{}", err)
                                }));
                            }
                        }

                        let cookie = Cookie::build("refresh", token.to_string())
                            .path("/")
                            .http_only(true)
                            .secure(true)
                            .max_age(actix_web::cookie::time::Duration::hours(5))
                            .finish();

                         return HttpResponse::Ok()
                            .cookie(cookie)
                            .json(serde_json::json!({
                                "info": "Proceed to login"
                            }))
                    }
                    Err(err) => 
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "info": format!("Unable to parse password: {}", err)
                    })),
                }
            } else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "info": "Unable to get the data"
                }))
            }
        }
        Err(err) => 
        return HttpResponse::BadRequest().json(serde_json::json!({
            "info": format!("{}", err)
        }))
        }  
}


    },
      Err(err)=>{
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "info":format!("{}",err)
            })
        )
    }
    }}

pub async fn get_email(form:web::Json<EmailInfo>,pool:web::Data<PgPool>,mail:web::Data<SmtpTransport>,mail_from:web::Data<String>)->impl Responder{
    let info=form.into_inner();
    let email=info.email.clone();
    let result=sqlx::query!("SELECT email FROM student WHERE email=$1",email)
        .fetch_one(pool.get_ref())
        .await;
    match result{
        Ok(_)=>{
                let mut rng =rand::rng();
                let otp=rng.random_range(100000..999999);
                let duration=chrono::Utc::now();
                let result=sqlx::query!("UPDATE student SET forgotten_pass_otp=$1,forgotten_pass_createdAt=$2 WHERE email=$3",otp.to_string(),duration,email)
                .execute(pool.get_ref())
                .await;
                match result{
                    Ok(_)=>{
                        
                let msg=Message::builder()
                    .from(mail_from.as_str().parse().unwrap())
                    .to(email.parse().unwrap())
                    .subject("Sending Code for Password")
                    .body(format!("{}",otp))
                    .unwrap();
                let mail_cxt = mail.get_ref().clone();
                let send_mail = web::block(move || mail_cxt.send(&msg)).await;
                match send_mail{
                    Ok(_)=>{},
                    Err(err)=>{
                        return HttpResponse::BadRequest().json(
                            serde_json::json!(
                                {"info":format!("{}",err)}
                            )
                        )
                    }
                }
                        return HttpResponse::Ok().json(
                            serde_json::json!({
                                "info":"Added successfuly"
                            })
                        )
                    },
                    Err(err)=>{
                        return HttpResponse::BadRequest().json(
                            serde_json::json!({
                                "info":format!("{}",err)
                            })
                        )
                    }
                }

            // return HttpResponse::Ok().json(
            //     serde_json::json!({
            //         "info":"Info sent"
            //     })
            // )
            // return HttpResponse::Found().json(
            //     serde_json::json!({
            //         "info":"Email found"
            //     })
            // )

        },
        Err(err)=>{
            return HttpResponse::NotFound().json(
                serde_json::json!({
                    "info":format!("{}",err)
                })
            )
        }
    }

}





pub async fn forgotten_password(pool:web::Data<PgPool>,query_email:web::Json<EmailInfo>,form:web::Json<ForgottenPassword>)->impl Responder{

    let email=&query_email.email;
    let otp_user=form.otp.clone();
    let new_password=form.password.clone();
    let result=sqlx::query!(
        "SELECT forgotten_pass_otp,forgotten_pass_createdAt FROM student WHERE email=$1",
        email
    )
    .fetch_one(pool.get_ref())
    .await;

    match  result{
        Ok(val)=>{
            let otp=val.forgotten_pass_otp;
            let created_time=val.forgotten_pass_createdat.unwrap();
            
            if  otp.is_some(){
                let otp=otp.unwrap();
                let now=chrono::Utc::now();
        
                let diff_time=now-created_time;
                if diff_time.num_minutes() >5{
                    return HttpResponse::Ok().json(
                        serde_json::json!({
                            "info":"Otp expired pls request a new one"
                        })
                    )
                }
                if otp.to_string() == otp_user{
                    let hashed_password=bcrypt::hash(new_password,DEFAULT_COST);
                    match hashed_password{
                        Ok(val)=>{
                        let stored=sqlx::query!(
                        "UPDATE student SET password=$1 WHERE email=$2",
                        val,email
                    )
                    .execute(pool.get_ref())
                    .await;
                match stored {
                    Ok(_)=>{
                        return HttpResponse::Ok().json(
                            serde_json::json!(
                                {
                                    "info":"Password Reset Sucessfull"
                                }
                            )
                        )
                    },
                    Err(err)=>{
                        return HttpResponse::BadRequest().json(
                            serde_json::json!(
                                {
                                    "info":format!("{}",err)
                                }
                            )
                        )
                    }
                    
                }
                        },
                        Err(err)=>{
                            return HttpResponse::BadRequest().json(
                                serde_json::json!({
                                    "info":format!("{}",err)
                                })
                            )
                        }
                    }
                   
                
                
                }
                else{
                    return HttpResponse::NonAuthoritativeInformation().json(
                        serde_json::json!({
                            "info":"Unable to validate the Otp ,Wrong OTP"
                        })
                    )
                }
            }
            else{
                return HttpResponse::BadRequest().json(
                    serde_json::json!({
                        "info":"Unable to get the Otp"
                    })
                )
            }
           
            
            

        },
        Err(err)=>{
            return HttpResponse::InternalServerError().json(
                serde_json::json!({
                    "info":format!("{}",err)
                })
            )
        }
    }
}
 