#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
    };

    use write_file::SendFile;

    fn simple_std(content: &str) {
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
            stream.write_file(file).unwrap();
            stream.flush().unwrap();
        }

        handle.join().unwrap();
    }

    #[test]
    fn simple_std_cases() {
        simple_std("hello world!");
        simple_std("hello world!".repeat(100).as_str());
        simple_std("hello world!".repeat(10000).as_str());
    }
}
