use futures::stream::{self, StreamExt};
use std::io::{self, Write};
use tokio::process::Command;

#[tokio::main]
async fn main() -> io::Result<()> {
    let services = ["dnscrypt-proxy", "opera-proxy", "zapret2"];

    let status_stream = stream::iter(services)
        .map(|service| async move {
            let status = Command::new("systemctl")
                .args(["--quiet", "is-active", service])
                .status()
                .await;

            let active = match status {
                Ok(s) => s.success(),
                Err(_) => false,
            };

            (service, active)
        })
        .buffer_unordered(3);

    let mut results = Vec::with_capacity(services.len());
    let mut stream = status_stream;

    while let Some(result) = stream.next().await {
        results.push(result);
    }

    results.sort_by_key(|(service, _)| *service);

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Проверяем, поддерживает ли терминал цвета
    let use_colors = std::io::IsTerminal::is_terminal(&stdout);

    if use_colors {
        let _ = handle.write_all(b"\n\x1b[36m   --- SYSTEM STATUS ---\x1b[0m\n");
    } else {
        let _ = handle.write_all(b"\n   --- SYSTEM STATUS ---\n");
    }

    for (service, active) in results {
        if use_colors {
            if active {
                let _ = writeln!(
                    handle,
                    " \x1b[32m\u{2714}\x1b[0m {:<15} : \x1b[32mRUNNING\x1b[0m",
                    service
                );
            } else {
                let _ = writeln!(
                    handle,
                    " \x1b[31m✗\x1b[0m {:<15} : \x1b[31mSTOPPED\x1b[0m",
                    service
                );
            }
        } else {
            let status = if active { "RUNNING" } else { "STOPPED" };
            let _ = writeln!(handle, "   {:<15} : {}", service, status);
        }
    }

    if use_colors {
        let _ = handle.write_all(b"\x1b[36m----------------------------\x1b[0m\n\n");
    } else {
        let _ = handle.write_all(b"----------------------------\n\n");
    }

    let _ = handle.flush();

    Ok(())
}
