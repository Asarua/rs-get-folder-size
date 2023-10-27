use std::{future::Future, path::PathBuf, pin::Pin};
use clap::Parser;
use tokio::fs;

#[derive(Parser)]
#[command(name = "rgfs", version = env!("CARGO_PKG_VERSION"), bin_name = "rgfs")]
struct Command;

#[tokio::main]
async fn main() {
  let root = match std::env::var("root") {
    Ok(root) => PathBuf::from(root),
    _ => std::env::current_dir().expect("Fail to get current_dir."),
  };
  let folder_size = check_size(root).await;
  println!("{folder_size}");
}

#[inline]
fn check_size(path: PathBuf) -> Pin<Box<dyn Future<Output = usize> + Send + 'static>> {
  Box::pin(async move {
    match fs::metadata(path.clone()).await {
      Ok(metadata) => {
        if metadata.is_dir() {
          let mut dirs = fs::read_dir(path.clone()).await.expect(
            format!(
              "read {} dir fail.",
              path.clone().to_str().unwrap_or_default()
            )
            .as_str(),
          );
          let mut cur_dir_size = 0;
          while let Ok(Some(entry)) = dirs.next_entry().await {
            let block = tokio::spawn(async move { check_size(entry.path()).await });
            cur_dir_size += block.await.unwrap_or_default();
          }
          return cur_dir_size;
        } else if metadata.is_file() {
          return metadata.len() as usize;
        }
        0
      }
      Err(_) => 0,
    }
  })
}
