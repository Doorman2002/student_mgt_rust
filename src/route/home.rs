use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::{self, PgPool};
use serde_json::json;

pub async fn home_page(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // Extract cookie safely
    let cookie_opt = req.cookie("refresh");
    if cookie_opt.is_none() {
        return HttpResponse::NonAuthoritativeInformation().json(json!({"info": "Unable to Parse cookie"}));
    }
    let cookie = cookie_opt.unwrap().value().to_string();

    let user_result = sqlx::query!("SELECT token_id, course FROM student WHERE token_id = $1", cookie)
        .fetch_one(pool.get_ref())
        .await;

    match user_result {
        Ok(student) => {
            // Converts Option to Result. If None, the closure returns an HTTP Response.
            let token = match student.token_id.ok_or_else(|| HttpResponse::BadRequest().json(json!({"info": "Missing token"}))) {
                Ok(t) => t,
                Err(http_err) => return http_err, // Exits home_page early with the HTTP error
            };

            let course = match student.course.ok_or_else(|| HttpResponse::BadRequest().json(json!({"info": "No course assigned"}))) {
                Ok(c) => c,
                Err(http_err) => return http_err,
            };

            if token != cookie {
                return HttpResponse::BadRequest().json(json!({"info": "Access Denied"}));
            }

            let pdf_file = match course.as_str() {
                "code/programming" => "codeCourse.pdf",
                "ui/ux" => "Ui/Ux.pdf",
                "VideoEdit" => "videoedit.pdf",
                "digitalMarketing" => "digitalM.pdf",
                "productMgt" => "productMgt.pdf",
                "graphics" => "graphics.pdf",
                _ => return HttpResponse::BadRequest().json(json!({"info": "Unknown course"})),
            };

            let course_folder = format!("./course/{}", pdf_file);
            HttpResponse::Ok().json(json!({ "path": course_folder }))
        }
        Err(err) => {
            HttpResponse::NonAuthoritativeInformation().json(json!({"info": format!("{}", err)}))
        }
    }
}
