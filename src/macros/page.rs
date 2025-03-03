macro_rules! page {
    ($state:expr, $page_path:expr) => {{
        let page = match $state
            .tera
            .render($page_path, &tera::Context::new())
            .map(Html)
        {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrResponse {
                        error: e.to_string(),
                    })
                    .into_response(),
                );
            }
        };
        (StatusCode::OK, page.into_response())
    }};

    ($state:expr, $page_path:expr,$struct:expr) => {{
        let context = match tera::Context::from_serialize($struct) {
            Ok(c) => c,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrResponse {
                        error: e.to_string(),
                    })
                    .into_response(),
                );
            }
        };

        let page = match $state.tera.render($page_path, &context).map(Html) {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrResponse {
                        error: e.to_string(),
                    })
                    .into_response(),
                );
            }
        };
        (StatusCode::OK, page.into_response())
    }};
}

pub(crate) use page;
