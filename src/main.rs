use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use dotenvy;
use sqlx::postgres::PgPoolOptions;
mod models;
mod route;
use lettre::{SmtpTransport, transport::smtp::authentication::Credentials};
#[derive(Clone)]
pub struct Credential{
   pub info:Credentials
}
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Welcome ")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let mail_user=std::env::var("mail_username").expect("unable to get the username");
    let mail_pass=std::env::var("mail_password").expect("Unable to get the password");
    let cred_info=Credentials::new(mail_user,mail_pass);
    let cred=Credential{info:cred_info.clone()};
    let mailer=SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(cred_info)
        .build();

    let db = std::env::var("DATABASE_URL").expect("Unable to connect the DB");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db)
        .await
        .expect("Unable to open the Db");

    println!("Connecttion initiated");
    HttpServer::new(move || {
        let cors_headers = Cors::default()
            .allowed_origin("http://localhost:8004")
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::ACCEPT, http::header::AUTHORIZATION,http::header::CONTENT_TYPE])
            .supports_credentials()
            .max_age(3600 * 2);

        App::new()
            .app_data(web::Data::new(mailer.clone()))
            .app_data(web::Data::new(cred.clone()))
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors_headers)
            .route("/", web::get().to(hello))
            .route("/auth/signup",web::post().to(route::student::student_signup))
            .route("/auth/verify/email",web::post().to(route::student::verify_email))
            .route("/auth/login",web::post().to(route::student::login))
            .route("/forgottenpassword/get/email", web::get().to(route::student::get_email))
            .route("/forgottenpassword/post/otp",web::post().to(route::student::forgotten_password))
    })
    .bind("127.0.0.1:8004")?
    .run()
    .await
}
