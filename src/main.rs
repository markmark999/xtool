use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream, UdpSocket},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use serialport::SerialPort;
use tracing_subscriber::{fmt::MakeWriter, prelude::*};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

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
    /// does testing for udp
    Udp {
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
        Commands::Tcp { ip, port, send } => (
            Box::new(TcpStream::connect((ip, port))?) as Box<dyn Transport>,
            send,
        ),
        Commands::Udp { ip, port, send } => {
            todo!()
        }
        Commands::Serial {
            port,
            baudrate,
            send,
        } => (Box::new(serialport::new(port, baudrate).open()?), send),
    };

    if let Some(send) = args.1 {
        args.0.write_all(&send)?;
    }
    anyhow::Result::Ok(())
}
