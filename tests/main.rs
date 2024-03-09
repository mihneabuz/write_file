#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
    };

    use write_file::{SendFile, MAX_CHUNK_SIZE};

    fn simple_std(content: &str, chunk_size: usize) {
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let expected = String::from(content);
        let handle = std::thread::spawn(move || {
            let mut conn = TcpStream::connect(("0.0.0.0", port)).unwrap();

            let mut buf = Vec::new();
            conn.read_to_end(&mut buf).unwrap();

            assert_eq!(String::from_utf8(buf).unwrap().as_str(), expected);
        });

        let mut file = tempfile::tempfile().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        if let Ok((mut stream, _)) = listener.accept() {
            if chunk_size == 0 {
                stream.write_file_all(&file).unwrap();
            } else {
                stream.write_file_all_chunked(&file, chunk_size).unwrap();
            }

            stream.flush().unwrap();
        }

        handle.join().unwrap();
    }

    #[test]
    fn simple_std_cases() {
        let content = [
            "hello world!".to_string(),
            "hello world!".repeat(100),
            "hello world!".repeat(10000),
        ];

        let chunk_sizez = [0, 10, 1000, MAX_CHUNK_SIZE];

        for content in content {
            for chunk_size in chunk_sizez {
                simple_std(&content, chunk_size);
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tokio_tests {
    use std::env;

    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use write_file::{tokio::*, MAX_CHUNK_SIZE};

    async fn tokio_tmpfile(content: &str) -> File {
        let tmp_name = env::temp_dir().join("tokio-test");
        {
            File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_name)
                .await
                .unwrap()
                .write_all(content.as_bytes())
                .await
                .unwrap();
        }

        File::open(&tmp_name).await.unwrap()
    }

    async fn async_tokio(content: String, chunk_size: usize) {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let expected = content.clone();
        let handle = tokio::spawn(async move {
            let mut conn = TcpStream::connect(("0.0.0.0", port)).await.unwrap();

            let mut buf = Vec::new();
            conn.read_to_end(&mut buf).await.unwrap();

            assert_eq!(String::from_utf8(buf).unwrap().as_str(), expected);
        });

        let file = tokio_tmpfile(&content).await;

        if let Ok((mut stream, _)) = listener.accept().await {
            if chunk_size == 0 {
                stream.write_file_all(&file).await.unwrap();
            } else {
                stream
                    .write_file_all_chunked(&file, chunk_size)
                    .await
                    .unwrap();
            }
        }

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn async_tokio_cases() {
        let content = [
            "hello world!".to_string(),
            "hello world!".repeat(100),
            "hello world!".repeat(10000),
        ];

        let chunk_sizez = [0, 10, 1000, MAX_CHUNK_SIZE];

        for content in content {
            for chunk_size in chunk_sizez {
                async_tokio(content.clone(), chunk_size).await;
            }
        }
    }
}
