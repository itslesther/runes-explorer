use std::fs::OpenOptions;
use std::io::Write;

// #[derive(Debug, Clone, Copy)]
pub struct LogFile {
    data_file: std::fs::File,
}

impl LogFile {
    pub fn new() -> LogFile {
        LogFile {
            data_file: OpenOptions::new()
                .append(true)
                .open("indexer.log")
                .expect("cannot open file"),
        }
    }

    pub fn write(&mut self, data: &str) {
        self.data_file.write(data.as_bytes()).expect("write failed");

        self.data_file
            .write(("\n").as_bytes())
            .expect("write failed");

        println!("{}", data)
    }
}
