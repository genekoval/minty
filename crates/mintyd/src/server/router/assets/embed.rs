use super::{assets, Router};

use axum::{http::header::CONTENT_TYPE, routing::get};

macro_rules! asset {
    ($file:literal) => {
        include_bytes!(concat!(assets!(), $file))
    };
}

macro_rules! serve {
    ($(($file:literal, $ty:literal)),* $(,)?) => {
        Router::new()
            $(.route(
                $file,
                get(|| async {
                    ([(CONTENT_TYPE, $ty)], asset!($file))
                }),
            ))*
    };
}

pub fn routes() -> Router {
    serve! {
        ("/index.css", "text/css"),
        ("/index.js", "application/javascript"),
        ("/fonts/RobotoFlex-VariableFont_GRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght.ttf", "font/ttf"),
    }
}
