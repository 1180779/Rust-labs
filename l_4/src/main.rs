use std::io::{Read, Write};
use std::str::FromStr;
use std::{hint, time};

fn divisors(n: std::num::NonZero<u32>) -> std::collections::BTreeSet<std::num::NonZero<u32>> {
    let u = n.get();
    let mut result = std::collections::BTreeSet::<std::num::NonZero<u32>>::new();

    for i in 1..=n.isqrt().into() {
        if u.is_multiple_of(i) {
            result.insert(std::num::NonZeroU32::new(i).expect("i is guaranteed to be non-zero"));
            result.insert(
                std::num::NonZeroU32::new(u / i)
                    .expect("n / i is guaranteed to be non-zero since i and n are non zero"),
            );
        }
    }
    result
}

fn assert_sorted(buf: &[i32]) {
    // sorted:      [1, 2, 3, 4]
    // sorted:      [1, 1, 3, 3]
    // not sorted:  [2, 1, 2, 3]
    for w in buf.windows(2) {
        if w[0] > w[1] {
            panic!("slice is not sorted");
        }
    }
}

fn divisors_benchmark_single(n: std::num::NonZeroU32) -> time::Duration {
    let start = time::Instant::now();
    _ = hint::black_box(divisors(n));
    let end = time::Instant::now();
    end - start
}

fn divisors_benchmark() {
    let mut sum = time::Duration::new(0, 0);
    for i in 1..=100 {
        sum += hint::black_box(divisors_benchmark_single(
            std::num::NonZeroU32::new(i).expect("i is guaranteed to be non-zero"),
        ));
    }
    let avg = sum / 100;
    println!(
        "avg time is {}.{}ms ({:?})",
        avg.as_millis(),
        avg.as_micros(),
        avg
    );
}

fn bulk_write(stream: &mut std::net::TcpStream, buf: &[u8]) -> std::io::Result<()> {
    let mut written = 0;
    let mut count = 1;
    while count > 0 {
        let res = stream.write(&buf[written..buf.len()]);
        match res {
            Ok(written_this_time) => {
                written += written_this_time;
                count = written_this_time;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn bulk_read(stream: &mut std::net::TcpStream, size: usize) -> std::io::Result<Vec<u8>> {
    let mut adapter = stream.take(size.try_into().unwrap());

    const BUFF_SIZE: usize = 1024;
    let mut buffer: [u8; BUFF_SIZE] = [0; BUFF_SIZE];
    let mut count = 1;
    let mut vec = Vec::<u8>::new();
    while count > 0 {
        let res = adapter.read(&mut buffer);
        match res {
            Ok(read_this_time) => {
                vec.append(&mut buffer[0..read_this_time].to_vec());
                count = read_this_time;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(vec)
}

fn handle_client(stream: std::net::TcpStream) -> std::io::Result<()> {
    let mut stream: std::net::TcpStream = stream;

    /* read how many characters to read first (8 + 1 = 8 bytes for the path length + newline) */
    let len_bytes = bulk_read(&mut stream, 4)?;
    let len_str = match String::from_utf8(len_bytes) {
        Ok(s) => s,
        Err(e) => {
            let msg = "Bad protocol (len not utf8)\n";
            bulk_write(&mut stream, msg.as_bytes())?;
            println!("Error: {}", e);
            return Ok(());
        }
    };
    let len = match len_str.parse::<usize>() {
        Ok(n) => n,
        Err(e) => {
            let msg = "Bad protocol (len not a number)\n";
            bulk_write(&mut stream, msg.as_bytes())?;
            println!("Error: {}. Received len string: '{}'", e, len_str);
            return Ok(());
        }
    };

    /* read the path (+2 newline characters, which will be discarded) */
    let bytes = bulk_read(&mut stream, len + 2)?;
    println!("Read: {:?}", bytes);
    let path = String::from_utf8(bytes);
    match path {
        Ok(path) => {
            let path = path.trim();
            let path = std::path::PathBuf::from_str(path);
            match path {
                Ok(path) => {
                    let dir = std::fs::read_dir(path);
                    match dir {
                        Ok(dir) => {
                            let files = dir
                                .map(|d| {
                                    let str = d.unwrap()
                                        .path()
                                        .into_os_string()
                                        .into_string();
                                    let Ok(str) = str else {
                                        return String::new();
                                    };
                                    str
                                })
                                .fold(String::new(), |acc, s| if !s.is_empty() { acc + &s + "\n" } else { acc });
                            println!("files = {}", files);
                            bulk_write(&mut stream, files.as_bytes())?;
                        }
                        Err(e) => {
                            let msg = "Bad path\n".to_string();
                            bulk_write(&mut stream, msg.as_bytes())?;
                            println!("Error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let msg = "Bad dir\n".to_string();
                    bulk_write(&mut stream, msg.as_bytes())?;
                    println!("Error: {}", e);
                }
            }
        }
        Err(_) => {
            let msg = format!("Bad path\n({})", 1);
            bulk_write(&mut stream, msg.as_bytes())?;
            return Ok(());
        }
    }
    Ok(())
}

fn main() {
    println!(
        "divisors of 30: {:?}",
        divisors(std::num::NonZero::new(30).expect("30 is non zero"))
    );

    /* sorted assert */
    let sorted = [1, 2, 3, 4];
    // this one panics
    // let not_sorted = [2, 1, 3, 4];

    assert_sorted(&sorted);
    // assert_sorted(&not_sorted);

    /* divisors */
    divisors_benchmark();

    /* TCP server */
    println!("Listening on port: 7878");
    let listener = std::net::TcpListener::bind("127.0.0.1:7878").unwrap();
    for stream in listener.incoming() {
        let res = handle_client(stream.unwrap());
        if let Err(err) = res {
            println!("Error: {}", err);
        }
    }
}
