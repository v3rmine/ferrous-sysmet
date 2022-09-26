#[macro_export]
macro_rules! static_files_server {
    ($name:ident, $dir:ident, $content_type:expr) => {
        mod $name {
            use axum::{
                body::{self, Empty, Full},
                extract::Path,
                http::{header, HeaderValue, Response, StatusCode},
                response::IntoResponse,
            };

            pub async fn $name(Path(path): Path<String>) -> impl IntoResponse {
                let path = path.trim_start_matches('/');

                match super::$dir.get_file(path) {
                    None => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(body::boxed(Empty::new()))
                        .unwrap(),
                    Some(file) => Response::builder()
                        .status(StatusCode::OK)
                        .header(
                            header::CONTENT_TYPE,
                            HeaderValue::from_str($content_type).unwrap(),
                        )
                        .body(body::boxed(Full::from(file.contents())))
                        .unwrap(),
                }
            }
        }
        use $name::$name;
    };
}
