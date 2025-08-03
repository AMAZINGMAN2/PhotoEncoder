use actix_web::{web, HttpResponse, Error};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use bytes::BytesMut;

use crate::stego;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/encode").route(web::post().to(encode_handler))
    )
    .service(
        web::resource("/decode").route(web::post().to(decode_handler))
    );
}

async fn encode_handler(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut img_bytes = None;
    let mut secret_bytes = None;
    let mut password_bytes = None;

    while let Some(field) = payload.next().await {
        let mut field = field?;

        let name = field
            .content_disposition()
            .get_name()
            .map(|n| n.to_string())
            .unwrap_or_default();

        let mut data = BytesMut::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            data.extend_from_slice(&chunk);
        }

        match name.as_str() {
            "image" => img_bytes = Some(data.freeze()),
            "secret" => secret_bytes = Some(data.freeze()),
            "password" => password_bytes = Some(data.freeze()),
            _ => {}
        }
    }

    let img_bytes = match img_bytes {
        Some(b) => b,
        None => return Ok(HttpResponse::BadRequest().body("Missing image file")),
    };
    let secret_bytes = match secret_bytes {
        Some(b) => b,
        None => return Ok(HttpResponse::BadRequest().body("Missing secret data")),
    };

    let password_opt = password_bytes.as_ref().map(|b| b.as_ref());

    match stego::encode_image(&img_bytes, &secret_bytes, password_opt) {
        Ok(encoded_img_bytes) => Ok(HttpResponse::Ok()
            .content_type("image/png")
            .body(encoded_img_bytes)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Encoding error: {}", e))),
    }
}

async fn decode_handler(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut img_bytes = None;
    let mut password_bytes = None;

    while let Some(field) = payload.next().await {
        let mut field = field?;

        let name = field
            .content_disposition()
            .get_name()
            .map(|n| n.to_string())
            .unwrap_or_default();

        let mut data = BytesMut::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            data.extend_from_slice(&chunk);
        }

        match name.as_str() {
            "image" => img_bytes = Some(data.freeze()),
            "password" => password_bytes = Some(data.freeze()),
            _ => {}
        }
    }

    let img_bytes = match img_bytes {
        Some(b) => b,
        None => return Ok(HttpResponse::BadRequest().body("Missing image file")),
    };

    let password_opt = password_bytes.as_ref().map(|b| b.as_ref());

    match stego::decode_image(&img_bytes, password_opt) {
        Ok(secret) => Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(secret)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Decoding error: {}", e))),
    }
}
