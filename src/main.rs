use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use serialport::SerialPort;
use tracing_subscriber::prelude::*;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing for tcp
    Tcp {
        /// ipv4 address
        #[arg(short, long)]
        ip: Ipv4Addr,
        /// port number
        #[arg(short, long)]
        port: u16,
        /// send raw values (hex)
        #[arg(short, long, value_parser=from_hex,num_args = 1.., value_delimiter = ' ')]
        send: Option<Vec<u8>>,
    },
    /// does testing for serialport
    Serial {
        /// serial port name
        #[arg(short, long)]
        port: String,
        /// serial baudrate
        #[arg(short, long)]
        baudrate: u32,
        /// send raw values (hex)
        #[arg(short, long, value_parser=from_hex,num_args = 1.., value_delimiter = ' ')]
        send: Option<Vec<u8>>,
    },
}

fn from_hex(src: &str) -> Result<u8, String> {
    u8::from_str_radix(src, 16).map_err(|e| e.to_string())
}

trait Transport: Read + Write {}

impl Transport for TcpStream {}

impl Transport for Box<dyn SerialPort> {}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(tracing_appender::rolling::hourly("log", "xtool.log")),
        )
        .init();
    let cli = Cli::parse();

    let mut args: (Box<dyn Transport>, Option<Vec<u8>>) = match cli.command {
        Commands::Tcp { ip, port, send } => {
            let ts = Box::new(TcpStream::connect((ip, port))?);
            ts.set_read_timeout(Some(std::time::Duration::from_millis(100)))?;
            (ts, send)
        }

        Commands::Serial {
            port,
            baudrate,
            send,
        } => {
            let mut port = Box::new(serialport::new(port, baudrate).open()?);

            port.set_timeout(std::time::Duration::from_millis(500))?;
            (port, send)
        }
    };

    if let Some(send) = args.1 {
        args.0.write_all(&send)?;

        tracing::info!("sent: {:?}", &send);
    }

    let mut buffer = bytes::BytesMut::with_capacity(1024);
    loop {
        if args.0.read(&mut buffer).is_err() {
            break;
        }
    }
    tracing::info!("received: {:?}", &buffer);
    anyhow::Result::Ok(())
}
