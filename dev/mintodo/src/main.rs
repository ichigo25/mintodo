use std::time::Duration;
use std::file;

use tokio::runtime::Builder;
use tokio::net::TcpListener;
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

use tracing::{ Level, info, error };
use tracing_subscriber::FmtSubscriber;

fn main()
{
    // 設定値
    let app_name = "xxx";
    let worker_threads = 5;
    let blocking_threads = 50;
    let keep_alive = Duration::from_millis(60);
    let stack_size = 3145728;
    let address = "127.0.0.1";
    let port = "8000";

    // Tracingの設定
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc3339())
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Initializing {}", app_name);

    // Tokioのランタイム
    let runtime = match Builder::new_multi_thread()
        .enable_io()
        .worker_threads(worker_threads)
        .max_blocking_threads(blocking_threads)
        .thread_name(format!("{}-thread", app_name))
        .thread_keep_alive(keep_alive)
        .thread_stack_size(stack_size)
        .build()
        {
            Ok(runtime) => runtime,
            Err(e) =>
            {
                error!("runtime error: {}", e);
                return;
            },
        };

    runtime.block_on(async
    {
        // アドレスとポートをTcpListenerにバインディング
        let listener = match TcpListener::bind(format!("{}:{}", address, port)).await
        {
            Ok(listener) => listener,
            Err(e) =>
            {
                error!("tcp listener error: {}", e);
                return;
            },
        };
        info!("listening on: {:?}", listener);

        loop
        {
            // ソケットと接続先情報の取得
            let (mut socket, data) = match listener.accept().await
            {
                Ok((socket, data)) => (socket, data),
                Err(e) =>
                {
                    error!("application error: {}", e);
                    return;
                },
            };

            info!("accept: {}", data);

            tokio::spawn(async move
            {
                let mut buf = [0; 1024];

                loop
                {
                    // クライアントのリクエスト（ソケットのデータ）を書き出し
                    let _n = match socket.read(&mut buf).await
                    {
                        Ok(n) if n == 0 => return,  // ソケットがclose()の場合
                        Ok(n) => n,
                        Err(e) =>
                        {
                            error!("failed to read from socket: err = {}", e);
                            return;
                        },
                    };

                    let response = "test";

                    // クライアントへのレスポンス
                    if let Err(e) = socket.write_all(response.as_bytes()).await
                    {
                        error!("failed to write to socket: err = {}", e);
                        return;
                    }
                }
            });
        }
    })
}

