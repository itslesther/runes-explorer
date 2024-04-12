use anyhow::Error;
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
        self.data_file
            .write(&format!("{}\n", data).as_bytes())
            .expect("write failed");

        println!("{}", data);

        // self.data_file.flush().expect("flush failed");
    }

    pub fn close(&mut self) {
        self.data_file.flush().expect("flush failed");
    }
}

pub fn log(data: &str) -> Result<(), Error> {
    let mut data_file = OpenOptions::new().append(true).open("indexer.log")?;

    data_file.write(&format!("{}\n", data).as_bytes())?;

    data_file.flush()?;

    println!("{}", data);
    Ok(())
}
