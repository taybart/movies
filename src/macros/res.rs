macro_rules! res {
    ($condition:expr, $err:expr) => {{
        match $condition {
            Ok(x) => x,
            Err(e) => {
                error!("{e}");
                return $err;
            }
        }
    }};
}

pub(crate) use res;
