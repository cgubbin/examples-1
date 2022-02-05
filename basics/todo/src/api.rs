use actix_files::NamedFile;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{dev, error, http, web, Error, HttpResponse, Result};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use tera::{Context, Tera};

use crate::db;

pub async fn index(
    pool: web::Data<PgPool>,
    tmpl: web::Data<Tera>,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, Error> {
    let tasks = db::get_all_tasks(&pool)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut context = Context::new();
    context.insert("tasks", &tasks);

    //Session is set during operations on other endpoints
    //that can redirect to index
    for m in flash_messages.iter() {
        context.insert("msg", &(m.level(), m.content()));
    }

    let rendered = tmpl
        .render("index.html.tera", &context)
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(rendered))
}

#[derive(Deserialize)]
pub struct CreateForm {
    description: String,
}

pub async fn create(
    params: web::Form<CreateForm>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    if params.description.is_empty() {
        FlashMessage::error("Description cannot be empty").send();
        Ok(redirect_to("/"))
    } else {
        db::create_task(params.into_inner().description, &pool)
            .await
            .map_err(error::ErrorInternalServerError)?;
        FlashMessage::success("Task successfully added").send();
        Ok(redirect_to("/"))
    }
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Deserialize)]
pub struct UpdateForm {
    _method: String,
}

pub async fn update(
    db: web::Data<PgPool>,
    params: web::Path<UpdateParams>,
    form: web::Form<UpdateForm>,
) -> Result<HttpResponse, Error> {
    match form._method.as_ref() {
        "put" => toggle(db, params).await,
        "delete" => delete(db, params).await,
        unsupported_method => {
            let msg = format!("Unsupported HTTP method: {}", unsupported_method);
            Err(error::ErrorBadRequest(msg))
        }
    }
}

async fn toggle(
    pool: web::Data<PgPool>,
    params: web::Path<UpdateParams>,
) -> Result<HttpResponse, Error> {
    db::toggle_task(params.id, &pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(redirect_to("/"))
}

async fn delete(
    pool: web::Data<PgPool>,
    params: web::Path<UpdateParams>,
) -> Result<HttpResponse, Error> {
    db::delete_task(params.id, &pool)
        .await
        .map_err(error::ErrorInternalServerError)?;
    FlashMessage::success("Task was deleted.").send();
    Ok(redirect_to("/"))
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::Found()
        .insert_header((http::header::LOCATION, location))
        .finish()
}

pub fn bad_request<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = NamedFile::open("static/errors/400.html")?
        .set_status_code(res.status())
        .into_response(res.request())
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_response),
    ))
}

pub fn not_found<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let new_response = NamedFile::open("static/errors/404.html")?
        .set_status_code(res.status())
        .into_response(res.request())
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_response),
    ))
}

pub fn internal_server_error<B>(
    res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>> {
    let new_response = NamedFile::open("static/errors/500.html")?
        .set_status_code(res.status())
        .into_response(res.request())
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_response),
    ))
}
