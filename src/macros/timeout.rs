#[macro_export]
macro_rules! cancel {
    ($task:expr, $millis:expr) => {{
        use std::io::{Error, ErrorKind};
        use tokio::{
            select,
            time::{sleep,Duration},
        };

        let duration = Duration::from_millis($millis);

        select! {
            _ = async { sleep(duration).await } =>  Err(Error::new(ErrorKind::TimedOut, format!("timeout expired ({duration:?})"))),
            rel = $task => Ok(rel)
        }
    }};
}

#[macro_export]
macro_rules! handle {
    // 匹配一个表达式和模式
    ($result:expr, $err:pat => $err_block:expr) => {
        match $result {
            Ok(r) => r,
            Err($err) => $err_block,
        }
    };
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_cancel() {
        let a: Option<Result<&str, &str>> = cancel!(
            async {
                sleep(Duration::from_millis(9)).await;
                Some(Ok("i"))
            },
            10
        )
        .expect("[E] timeout");
        dbg!(a);
    }

    #[tokio::test]
    async fn test_handle() {
        for i in 0..9 {
            println!("i:{i}");
            let i = handle!(
                cancel!(async { sleep(Duration::from_millis(i)).await; i }, 5),
                e =>
                {
                    println!("[E] raw {i}timeout expired {e:?}");
                    continue;
                }
            );
            println!("iii:{i}");
        }
    }
}
