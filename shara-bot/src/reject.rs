#[macro_export]
macro_rules! reject_result {
    ($cx:expr, $result:expr, $error:expr, $message:expr, $state:expr) => {
        match $result {
            Ok(object) => object,
            Err(error) => {
                log::error!("{}: {}", $error, error);
                $cx.answer($message).await?;
                return next($state);
            }
        }
    };
}

#[macro_export]
macro_rules! reject_option {
    ($cx:expr, $option:expr, $message:expr, $state:expr) => {
        match $option {
            Some(object) => object,
            None => {
                $cx.answer($message).await?;
                return next($state);
            }
        }
    };
}
