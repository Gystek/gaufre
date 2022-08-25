/**
 * Copyright (C) 2022 Gustek <gustek@riseup.net>.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{
    self,
    prelude::{Read, Write},
    Result,
};
use std::net::TcpStream;
use std::path::Path;
use std::process::{exit, Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
use config::{
    CMD_PREFIX, COMMAND_BROWSER, COMMAND_IMAGE, COMMAND_TELNET, COMMAND_TEXT, DOWNLOAD_FOLDER,
};

const GAUFRE_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, PartialEq, Eq)]
enum EltType {
    TextFile,
    Directory,
    CCSONameServer,
    Error,
    BinHexMacintoshFile,
    DOSBinaryFile,
    UuencodedFile,
    FullTextSearchServer,
    TelnetTextSession,
    BinaryFile,
    MirrorServer,
    GIFFile,
    ImageFile,
    JPGFile,
    PNGFile,
    /* Sorry but no Telnet3270 */
    HTMLFile,
    InformationalMessage,
}

impl TryFrom<char> for EltType {
    type Error = io::Error;

    fn try_from(c: char) -> Result<Self> {
        match c {
            '0' => Ok(Self::TextFile),
            '1' => Ok(Self::Directory),
            '2' => Ok(Self::CCSONameServer),
            '3' => Ok(Self::Error),
            '4' => Ok(Self::BinHexMacintoshFile),
            '5' => Ok(Self::DOSBinaryFile),
            '6' => Ok(Self::UuencodedFile),
            '7' => Ok(Self::FullTextSearchServer),
            '8' => Ok(Self::TelnetTextSession),
            '9' => Ok(Self::BinaryFile),
            '+' => Ok(Self::MirrorServer),
            'g' => Ok(Self::GIFFile),
            'I' => Ok(Self::ImageFile),
            'p' => Ok(Self::PNGFile),
            'j' => Ok(Self::JPGFile),
            'h' => Ok(Self::HTMLFile),
            'i' => Ok(Self::InformationalMessage),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown item type: {}", c),
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct FsElement {
    elt_type: EltType,
    content: String,
    link: String,
    server: String,
    port: u16,
}

impl Display for FsElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.elt_type != EltType::InformationalMessage {
            write!(f, "\x1b[1m")?;
        }
        write!(f, "{}\x1b[0m", self.content)?;
        if self.elt_type == EltType::Directory {
            write!(f, "...")?;
        }

        Ok(())
    }
}

fn n_alpha(n: u16) -> (char, char) {
    let c1: char = ('a' as u8 + (n / 26) as u8) as char;
    let c2: char = ('a' as u8 + (n % 26) as u8) as char;

    (c1, c2)
}

fn alpha_nth((c1, c2): (char, char)) -> Option<u16> {
    if c1 > 'z' || c1 < 'a' {
        None
    } else if c2 > 'z' || c2 < 'a' {
        None
    } else {
        let d1 = (c1 as u8) - ('a' as u8);
        let d2 = (c2 as u8) - ('a' as u8);
        Some((d1 as u16) * 26 + (d2 as u16))
    }
}

fn display_elements<'a>(l: impl Iterator<Item = &'a FsElement>) {
    let mut n = 0;

    for x in l {
        if x.elt_type != EltType::InformationalMessage && x.elt_type != EltType::Error {
            let (c1, c2) = n_alpha(n);
            print!("\x1b[1;32m{}{}\x1b[0m ", c1, c2);
            n += 1;
        }
        if x.elt_type == EltType::Error {
            print!("\x1b[1;31mERROR: ");
        }
        println!("{}\x1b[0m", x);
    }
}

fn query_path(server: &str, port: u16, path: &str) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(format!("{}:{}", server, port))?;
    let mut buf = Vec::new();

    stream.write(format!("{}\r\n", path).as_bytes())?;
    stream.read_to_end(&mut buf)?;

    Ok(buf)
}

fn get_listing(server: &str, port: u16, path: &str) -> Result<Vec<FsElement>> {
    let buf = query_path(server, port, path)?;

    String::from_utf8(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "UTF8-invalid data"))?
        .split("\r\n")
        .filter(|&s| s != "." && !s.is_empty() /* Last after '.CRLF' */)
        .map(|stem| {
            if stem.len() < 8 {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Malformed listing element:\n  `{}'", stem),
                ))
            } else {
                let elt_type = EltType::try_from(stem.chars().nth(0).unwrap())?;
                /* No choice if I want to count */
                let components = stem[1..].split('\t').collect::<Vec<&str>>();

                if components.len() != 4 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Malformed listing element:\n  `{}'", components.join("\t")),
                    ))
                } else {
                    let port: u16 = components[3].parse().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Invalid port number: {}", components[3]),
                        )
                    })?;
                    Ok(FsElement {
                        elt_type,
                        content: components[0].to_string(),
                        link: components[1].to_string(),
                        server: components[2].to_string(),
                        port,
                    })
                }
            }
        })
        .collect::<Result<Vec<FsElement>>>()
}

fn reboot(
    host: &mut String,
    port: &mut u16,
    path: &mut String,
    history: &Vec<(String, u16, String)>,
    hp: usize,
    elements: &mut Vec<FsElement>,
) -> Result<()> {
    assert!(hp < history.len());

    let page = &history[hp];
    *host = page.0.to_string();
    *port = page.1;
    *path = page.2.to_string();

    *elements = get_listing(host, *port, path)?;

    display_elements(elements.iter());

    Ok(())
}

macro_rules! prompt {
    ($($arg:tt)*) => {{
        let mut line = String::new();

        print!($($arg)*);

        io::stdout().flush()?;
        io::stdin().read_line(&mut line)?;
        line.trim().to_string()
    }};
}

fn link(
    elt: FsElement,
    host: &mut String,
    port: &mut u16,
    path: &mut String,
    history: &mut Vec<(String, u16, String)>,
    hp: &mut usize,
    elements: &mut Vec<FsElement>,
) -> Result<()> {
    if elt.elt_type == EltType::Directory || elt.elt_type == EltType::MirrorServer {
        history.push((elt.server.clone(), elt.port, elt.link.clone()));
        *hp = history.len() - 1;

        reboot(host, port, path, history, *hp, elements)?;
    }

    display_elements(elements.iter());

    if elt.elt_type != EltType::Directory && elt.elt_type != EltType::MirrorServer {
        let content = if elt.elt_type == EltType::HTMLFile && elt.content.starts_with("URL:") {
            Vec::new()
        } else {
            query_path(&elt.server, elt.port, &elt.link)?
        };

        match elt.elt_type {
            EltType::TextFile => write_text(&elt.content, content)?,
            EltType::BinHexMacintoshFile | EltType::DOSBinaryFile | EltType::BinaryFile => {
                write_download(&elt.content, content)?;
            }
            EltType::CCSONameServer => {
                println!("CCSONameServer are only supported for legacy reasons.");
            }
            EltType::UuencodedFile => {
                let fname = match get_fname(&elt.content)? {
                    Some(s) => s,
                    None => {
                        println!("Cancelled");
                        return Ok(());
                    }
                };
                let (mut inf, _) = mktemp()?;
                inf.write_all(&content)?;

                print_status(
                    Command::new("uudecode")
                        .arg("-o")
                        .arg(fname)
                        .stdin(inf)
                        .status()?,
                );
            }
            EltType::FullTextSearchServer => {
                let search = prompt!("Enter your search string: ");

                history.push((
                    elt.server.clone(),
                    elt.port,
                    format!("{}\t{}", elt.link, search),
                ));
                *hp = history.len() - 1;

                reboot(host, port, path, history, *hp, elements)?;
            }
            EltType::TelnetTextSession => print_status(
                Command::new(COMMAND_TELNET)
                    .arg(elt.server)
                    .arg(elt.port.to_string())
                    .status()?,
            ),
            EltType::GIFFile | EltType::ImageFile | EltType::JPGFile | EltType::PNGFile => {
                let temp = || -> Result<()> {
                    let (mut f, fname) = mktemp()?;

                    f.write_all(&content)?;

                    print_status(Command::new(COMMAND_IMAGE).arg(&fname).status()?);
                    Ok(())
                };
                if is_download()? {
                    let fname = get_fname(&elt.content)?;
                    if let Some(fname) = fname {
                        File::create(fname)?.write_all(&content)?;
                    } else {
                        temp()?;
                    }
                } else {
                    temp()?;
                }
            }
            EltType::HTMLFile => {
                if elt.link.starts_with("URL:") {
                    print_status(Command::new(COMMAND_BROWSER).arg(&elt.link[4..]).status()?);
                } else {
                    let web_show = || -> Result<()> {
                        let (mut file, fname) = mktemp()?;

                        file.write_all(&content)?;

                        print_status(Command::new(COMMAND_BROWSER).arg(fname).status()?);

                        Ok(())
                    };

                    if is_download()? {
                        if let Some(fname) = get_fname(&elt.content)? {
                            File::create(&fname)?.write_all(&content)?;
                        } else {
                            web_show()?;
                        }
                    } else {
                        web_show()?;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn is_download() -> Result<bool> {
    Ok(prompt!("Do you want to download the file [y/N]? ").to_ascii_lowercase() == "y")
}

fn command(
    host: &mut String,
    port: &mut u16,
    path: &mut String,
    history: &mut Vec<(String, u16, String)>,
    hp: &mut usize,
    elements: &mut Vec<FsElement>,

    command: &str,
    args: &str,
) -> Result<()> {
    match command {
        "b" => {
            if *hp > 0 {
                *hp -= 1;
                reboot(host, port, path, history, *hp, elements)?;
            }
        }
        "f" => {
            if *hp + 1 < history.len() {
                *hp += 1;
                reboot(host, port, path, history, *hp, elements)?;
            }
        }
        "r" => {
            reboot(host, port, path, history, *hp, elements)?;
        }
        "s" => {
            if args.is_empty() {
                println!("{}:{}", *host, *port);
            } else {
                match parse_host(&args) {
                    Ok((h, p)) => {
                        *host = h;
                        *port = p;
                        *path = String::new();

                        *elements = get_listing(host, *port, path)?;
                        display_elements(elements.iter());
                    }
                    _ => {
                        eprintln!("Invalid server:\n  `{}'\n", args);
                    }
                }
            }
        }
        "q" => {
            println!("Goodbye.");
            exit(0);
        }
        _ => {
            println!(
                r#"gaufre -- version {}

Select a menu by typing the two letters in front of it (normally written
in bold green).

* List of commands

  Command prefix: {}

b             ; go back in the history
f             ; go forth in the history
s HOST[:PORT] ; change the current server and access it
r             ; reload the current page
q             ; exit the program
h             ; print this message"#,
                GAUFRE_VERSION, CMD_PREFIX
            );
        }
    }

    Ok(())
}

fn getline(
    host: &mut String,
    port: &mut u16,
    path: &mut String,
    history: &mut Vec<(String, u16, String)>,
    hp: &mut usize,
    elements: &mut Vec<FsElement>,
) -> Result<()> {
    let line = prompt!("\x1b[1m{}:{} {}>\x1b[0m ", host, port, path);

    /* Command handler */
    if line.chars().nth(0) == Some(CMD_PREFIX) {
        let (cmd, args) = line[1..].split_once(' ').unwrap_or((&line[1..], ""));

        command(host, port, path, history, hp, elements, cmd, args)
    } else if line == "help" {
        command(host, port, path, history, hp, elements, "h", "")
    } else {
        if line.len() != 2 {
            return Ok(());
        }
        let mut chars = line.chars();

        let chrs = (chars.next().unwrap(), chars.next().unwrap());

        let id = match alpha_nth(chrs) {
            Some(x) => x,
            None => return Ok(()),
        };

        match elements
            .iter()
            .filter(|e| e.elt_type != EltType::InformationalMessage)
            .nth(id as usize)
        {
            Some(e) => link(e.clone(), host, port, path, history, hp, elements),
            None => Ok(()),
        }
    }
}

fn print_status(s: ExitStatus) {
    if !s.success() {
        println!(
            "Command failed with exit code {}",
            match s.code() {
                Some(x) => x.to_string(),
                None => "(killed)".to_string(),
            }
        );
    } else {
        println!("Command finished successfully");
    }
}

fn mktemp() -> Result<(File, String)> {
    let mut now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut fname = format!("/tmp/tmp.{}", now);

    while Path::new(&fname).exists() {
        now += 1;
        fname = format!("/tmp/tmp.{}", now);
    }

    Ok((File::create(&fname)?, fname))
}

fn get_fname(name: &str) -> Result<Option<String>> {
    let link = match DOWNLOAD_FOLDER {
        None => prompt!("Where should the file be saved (empty to cancel)? "),
        Some(folder) => {
            if name.is_empty() {
                format!(
                    "{}/{}",
                    folder,
                    prompt!("Please enter a filename (empty to cancel)? ")
                )
            } else {
                format!("{}/{}", folder, name)
            }
        }
    };

    if link.is_empty() {
        return Ok(None);
    }

    if Path::new(&link).exists() {
        if prompt!("File `{}' already exists. Replace it [y/N]?", &link).to_ascii_lowercase() == "y"
        {
            Ok(Some(link))
        } else {
            return get_fname(name);
        }
    } else {
        Ok(Some(link))
    }
}

fn write_download(name: &str, content: Vec<u8>) -> Result<String> {
    let link = match get_fname(name)? {
        Some(s) => s,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Cancelled")),
    };

    File::create(&link)?.write_all(&content)?;

    println!("File saved.");

    Ok(link)
}

fn write_text(fname: &str, b: Vec<u8>) -> Result<()> {
    let download_it = || -> Result<()> {
        if let Some(fname) = get_fname(fname)? {
            File::create(fname)?.write_all(&b)
        } else {
            Ok(())
        }
    };
    if COMMAND_TEXT.is_none() {
        if is_download()? {
            download_it()
        } else {
            let s = String::from_utf8(b)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "UTF8-invalid data"))?;

            println!("{}", s);

            Ok(())
        }
    } else {
        let (mut file, _) = mktemp()?;
        file.write_all(&b)?;

        print_status(Command::new(COMMAND_TEXT.unwrap()).stdin(file).status()?);

        if is_download()? {
            download_it()
        } else {
            Ok(())
        }
    }
}

fn parse_host(host: &str) -> Result<(String, u16)> {
    if host.contains(':') {
        let (host, r_port) = host.split_once(':').unwrap();

        r_port
            .parse()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid port number: {}", r_port),
                )
            })
            .map(|x| (host.to_string(), x))
    } else {
        Ok((host.to_string(), 70))
    }
}

fn try_main() -> Result<()> {
    let (mut host, mut port) = std::env::args().nth(1).map_or(
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Usage: gaufre HOST[:PORT]",
        )),
        |x| parse_host(&x),
    )?;
    let mut path = String::new();
    let mut history = vec![(host.clone(), port, path.clone())];
    let mut hp = 0;
    let mut elements = get_listing(&host, port, &path)?;

    display_elements(elements.iter());

    println!("\tWelcome to gaufre -- type `/h' for help");

    loop {
        match getline(
            &mut host,
            &mut port,
            &mut path,
            &mut history,
            &mut hp,
            &mut elements,
        ) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
    }
}

fn main() {
    match try_main() {
        Ok(_) => exit(0),
        Err(e) => println!("{}", e),
    }

    exit(1);
}
