use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
    process,
};

use colored::Colorize;

pub struct Entry {
    pub kaam_entry: String,
    pub done: bool,
}

impl Entry {
    pub fn new(kaam_entry: String, done: bool) -> Self {
        Self { kaam_entry, done }
    }

    pub fn file_line(&self) -> String {
        let symbol = if self.done { "[*] " } else { "[ ] " };
        format!("{}{}\n", symbol, self.kaam_entry)
    }

    pub fn list_line(&self, number: usize) -> String {
        let kaam_entry = if self.done {
            self.kaam_entry.strikethrough().to_string()
        } else {
            self.kaam_entry.clone()
        };
        format!("{}: {}\n", number.to_string().bold(), kaam_entry)
    }

    pub fn read_line(line: &String) -> Self {
        let done = &line[..4] == "[*] ";
        let kaam_entry = (&line[4..]).to_string();
        Self { kaam_entry, done }
    }

    pub fn raw_line(&self) -> String {
        format!("{}\n", self.kaam_entry)
    }
}

pub struct Kaam {
    pub kaam: Vec<String>,
    pub kaam_path: String,
    pub kaam_bak: String,
    pub no_backup: bool,
}

impl Kaam {
    pub fn new() -> Result<Self, String> {
        let kaam_path: String = match env::var("kaam_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").unwrap();
                let legacy_kaam = format!("{}/.kaam", &home);
                match Path::new(&legacy_kaam).exists() {
                    true => legacy_kaam,
                    false => format!("{}/.kaam", &home),
                }
            }
        };

        let kaam_bak: String = match env::var("kaam_BAK_DIR") {
            Ok(t) => t,
            Err(_) => String::from("/tmp/kaam_bak"),
        };

        let no_backup = env::var("kaam_NO_BACKUP").is_ok();

        let kaam_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&kaam_path)
            .expect("Couldn't open the kaam file");

        let mut buf_reader = BufReader::new(kaam_file);

        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents).unwrap();

        let kaam = contents.lines().map(str::to_string).collect();

        Ok(Self {
            kaam,
            kaam_path,
            kaam_bak,
            no_backup,
        })
    }

    pub fn list(&self) {
        let stdout = io::stdout();
        let mut writer = BufWriter::new(stdout);
        let mut data = String::new();

        for (number, task) in self.kaam.iter().enumerate() {
            let entry = Entry::read_line(task);
            let number = number + 1;
            let line = entry.list_line(number);
            data.push_str(&line);
        }
        writer
            .write_all(data.as_bytes())
            .expect("Failed to write to stdout");
    }

    pub fn raw(&self, arg: &[String]) {
        if arg.len() > 1 {
            println!("kaam raw takes only one argument");
        } else if arg.is_empty() {
            println!("kaam raw requires an argument");
        } else {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout);
            let mut data = String::new();

            let arg = &arg[0];
            for task in self.kaam.iter() {
                let entry = Entry::read_line(task);
                if entry.done && arg == "done" {
                    data = entry.raw_line();
                } else if !entry.done && arg == "kaam" {
                    data = entry.raw_line();
                }
                writer
                    .write_all(data.as_bytes())
                    .expect("Failed to write to stdout");
            }
        }
    }

    pub fn add_kaam(&self, args: &[String]) {
        if args.is_empty() {
            println!("kaam add requires an argument");
            process::exit(1);
        }
        let kaam_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.kaam_path)
            .expect("Couldn't open the kaam file");

        let mut buffer = BufWriter::new(kaam_file);
        for arg in args {
            if arg.trim().is_empty() {
                continue;
            }
            let entry = Entry::new(arg.to_string(), false);
            let line = entry.file_line();
            buffer
                .write_all(line.as_bytes())
                .expect("Failed to write to kaam file");
        }
    }

    pub fn remove(&self, args: &[String]) {
        if args.is_empty() {
            println!("kaam remove requires an argument");
            process::exit(1);
        }
        let kaamfle = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.kaam_path)
            .expect("Couldn't open the kaamfile");

        let mut buffer = BufWriter::new(kaamfle);
        for (pos, line) in self.kaam.iter().enumerate() {
            if args.contains(&(pos + 1).to_string()) {
                continue;
            }
            let line = format!("{}\n", line);

            buffer
                .write_all(line.as_bytes())
                .expect("unable to read the data.");
        }
    }

    fn remove_file(&self) {
        match fs::remove_file(&self.kaam_path) {
            Ok(_) => {}
            Err(e) => println!("Error removing kaam file: {}", e),
        }
    }

    pub fn reset(&self) {
        if !self.no_backup {
            match fs::copy(&self.kaam_path, &self.kaam_bak) {
                Ok(_) => self.remove_file(),
                Err(_) => print!("Couldn't backup the kaam file."),
            }
        } else {
            self.remove_file();
        }
    }
    pub fn restore(&self) {
        fs::copy(&self.kaam_bak, &self.kaam_path)
            .expect("Couldn't restore the kaam file from backup");
    }
    pub fn sort(&self) {
        let _new_kaam: String;
        let mut kaam = String::new();
        let mut done = String::new();

        for line in self.kaam.iter() {
            let entry = Entry::read_line(line);
            if entry.done {
                let line = format!("{}\n", line);
                done.push_str(&line);
            } else {
                let line = format!("{}\n", line);
                kaam.push_str(&line);
            }
        }

        let newkaam = format!("{}{}", &kaam, &done);
        let mut kaam_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.kaam_path)
            .expect("Couldn't open the kaam file");

        kaam_file
            .write_all(newkaam.as_bytes())
            .expect("Failed to write to kaam file");
    }

    pub fn done(&self, args: &[String]) {
        if args.is_empty() {
            println!("kaam done requires an argument");
            process::exit(1);
        }

        let kaam_file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&self.kaam_path)
            .expect("Couldn't open the kaam file");

        let mut buffer = BufWriter::new(kaam_file);
        let mut data = String::new();

        for (pos, line) in self.kaam.iter().enumerate() {
            let mut entry = Entry::read_line(line);
            let line = if args.contains(&(pos + 1).to_string()) {
                entry.done = !entry.done;
                entry.file_line()
            } else {
                format!("{}\n", line)
            };
            data.push_str(&line);
        }
        buffer
            .write_all(data.as_bytes())
            .expect("Failed to write to kaam file");
    }

    pub fn edit(&self, args: &[String]) {
        if args.is_empty() || args.len() != 2 {
            println!("kaam edit takes exactly 2 arguments.");
            process::exit(1);
        }

        let kaamfile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.kaam_path)
            .expect("Couldn't open the kaam file");
        let mut buffer = BufWriter::new(kaamfile);

        for (pos, line) in self.kaam.iter().enumerate() {
            let line = if args[0] == (pos + 1).to_string() {
                let mut entry = Entry::read_line(line);
                entry.kaam_entry = args[1].clone();
                entry.file_line()
            } else {
                format!("{}\n", line)
            };
            buffer
                .write_all(line.as_bytes())
                .expect("Failed to write to kaam file");
        }
    }
}
const KAAM_HELP: &str = "Usage: kaam [COMMAND] [ARGUMENTS]
kaam Aayush Daddy ne rust me banaya hai free time me apne, task toh manage nahi hote isse.
Jaise: kaam list
YE SAB USE KARNA HAI :-
    - add [TASK/s]
        adds new task/s
        Example: kaam add \"academic comeback\"
    - edit [INDEX] [EDITED TASK/s]
        edits an existing task/s
        Example: kaam edit 1 banana
    - list
        lists all tasks
        Example: kaam list
    - done [INDEX]
        marks task as done
        Example: kaam done 2 3 (marks second and third tasks as completed)
    - rm [INDEX]
        removes a task
        Example: kaam rm 4
    - reset
        deletes all tasks
    - restore 
        restore recent backup after reset
    - sort
        sorts completed and uncompleted tasks
        Example: kaam sort
    - raw [kaam/done]
        prints nothing but done/incompleted tasks in plain text, useful for scripting (in short chutiyagiri).
        Example: kaam raw done
";
pub fn help() {
    // For readability
    println!("{}", KAAM_HELP.green().bold());
}
