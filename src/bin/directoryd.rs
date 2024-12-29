use bitcoind::bitcoincore_rpc::Auth;
use coinswap::{
    market::directory::{start_directory_server, DirectoryServer, DirectoryServerError},
    utill::{parse_proxy_auth, setup_directory_logger, ConnectionType},
    wallet::RPCConfig,
};

#[cfg(feature = "tor")]
use coinswap::tor::setup_mitosis;
use std::{env, path::PathBuf, str::FromStr, sync::Arc};

fn parse_args() -> Result<(String, Option<PathBuf>, String, (String, String), String), String> {
    let mut args = env::args().skip(1); // Skip the executable name

    let mut network = "clearnet".to_string(); // Default value
    let mut data_directory: Option<PathBuf> = None;
    let mut rpc = "127.0.0.1:18443".to_string(); // Default value
    let mut auth = ("user".to_string(), "password".to_string()); // Default value
    let mut rpc_network = "regtest".to_string(); // Default value

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-n" | "--network" => {
                if let Some(value) = args.next() {
                    if value == "tor" || value == "clearnet" {
                        network = value;
                    } else {
                        return Err(format!("Invalid value for network: {}", value));
                    }
                } else {
                    return Err("Missing value for network".to_string());
                }
            }
            "-d" | "--data-directory" => {
                if let Some(value) = args.next() {
                    data_directory = Some(PathBuf::from(value));
                } else {
                    return Err("Missing value for data-directory".to_string());
                }
            }
            "-r" | "--rpc" => {
                if let Some(value) = args.next() {
                    rpc = value;
                } else {
                    return Err("Missing value for rpc".to_string());
                }
            }
            "-a" | "--auth" => {
                if let Some(value) = args.next() {
                    auth = parse_proxy_auth(&value).map_err(|e| e.to_string())?;
                } else {
                    return Err("Missing value for auth".to_string());
                }
            }
            "--rpc_network" => {
                if let Some(value) = args.next() {
                    if ["regtest", "signet", "mainnet"].contains(&value.as_str()) {
                        rpc_network = value;
                    } else {
                        return Err(format!("Invalid value for rpc_network: {}", value));
                    }
                } else {
                    return Err("Missing value for rpc_network".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
    }

    Ok((network, data_directory, rpc, auth, rpc_network))
}

fn main() -> Result<(), DirectoryServerError> {
    setup_directory_logger(log::LevelFilter::Info);

    let (network, data_directory, rpc, auth, rpc_network) = parse_args().map_err(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }).unwrap();

    let rpc_network = bitcoin::Network::from_str(&rpc_network).unwrap();

    let conn_type = ConnectionType::from_str(&network)?;

    let rpc_config = RPCConfig {
        url: rpc,
        auth: Auth::UserPass(auth.0, auth.1),
        network: rpc_network,
        wallet_name: "random".to_string(), // we can put anything here as it will get updated in the init.
    };

    #[cfg(feature = "tor")]
    {
        if conn_type == ConnectionType::TOR {
            setup_mitosis();
        }
    }
    let directory = Arc::new(DirectoryServer::new(data_directory, Some(conn_type))?);

    start_directory_server(directory, Some(rpc_config))?;

    Ok(())
}
