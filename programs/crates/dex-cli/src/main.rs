use clap::{Parser, Subcommand};
use dex_core::{
    find_leader_pda, find_registry_pda, find_subscription_pda, CopyAction,
    TradeRegistryInstruction,
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "dex-cli", about = "Encode trade-registry Solana instructions")]
struct Cli {
    #[arg(long, default_value = "TradeRegistry1111111111111111111111111111111")]
    program_id: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print registry PDA for a program id.
    PdaRegistry,
    /// Print leader profile PDA.
    PdaLeader {
        #[arg(long)]
        leader: String,
    },
    /// Print follower subscription PDA.
    PdaSubscription {
        #[arg(long)]
        follower: String,
        #[arg(long)]
        leader: String,
    },
    /// Emit base58-encoded instruction data for InitializeRegistry.
    EncodeInit,
    /// Emit instruction data for RegisterLeader.
    EncodeRegisterLeader {
        #[arg(long, default_value_t = 100)]
        max_followers: u32,
    },
    /// Emit instruction data for Subscribe.
    EncodeSubscribe {
        #[arg(long, default_value_t = 10_000)]
        size_bps: u16,
    },
    /// Emit instruction data for LogCopyIntent.
    EncodeLogCopyIntent {
        #[arg(long, default_value = "buy")]
        action: String,
        #[arg(long)]
        mint: String,
        #[arg(long, default_value_t = 50_000_000)]
        amount_lamports: u64,
        #[arg(long, default_value = "0")]
        reference_sig_hex: String,
    },
}

fn program_id(value: &str) -> Pubkey {
    Pubkey::from_str(value).expect("valid program id")
}

fn pubkey(value: &str) -> Pubkey {
    Pubkey::from_str(value).expect("valid pubkey")
}

fn hex_to_sig(input: &str) -> [u8; 64] {
    let trimmed = input.trim_start_matches("0x");
    if trimmed.is_empty() {
        return [0u8; 64];
    }
    let bytes = hex::decode(trimmed).expect("valid hex reference signature");
    assert_eq!(bytes.len(), 64, "reference signature must be 64 bytes");
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    out
}

fn print_instruction(instruction: TradeRegistryInstruction) {
    let bytes = borsh::to_vec(&instruction).expect("serialize instruction");
    println!("bytes_len={}", bytes.len());
    println!("hex={}", hex::encode(&bytes));
}

fn main() {
    let cli = Cli::parse();
    let program_id = program_id(&cli.program_id);

    match cli.command {
        Commands::PdaRegistry => {
            let (pda, bump) = find_registry_pda(&program_id);
            println!("pda={pda}");
            println!("bump={bump}");
        }
        Commands::PdaLeader { leader } => {
            let leader = pubkey(&leader);
            let (pda, bump) = find_leader_pda(&program_id, &leader);
            println!("pda={pda}");
            println!("bump={bump}");
        }
        Commands::PdaSubscription { follower, leader } => {
            let follower = pubkey(&follower);
            let leader = pubkey(&leader);
            let (pda, bump) = find_subscription_pda(&program_id, &follower, &leader);
            println!("pda={pda}");
            println!("bump={bump}");
        }
        Commands::EncodeInit => print_instruction(TradeRegistryInstruction::InitializeRegistry),
        Commands::EncodeRegisterLeader { max_followers } => {
            print_instruction(TradeRegistryInstruction::RegisterLeader { max_followers })
        }
        Commands::EncodeSubscribe { size_bps } => {
            print_instruction(TradeRegistryInstruction::Subscribe { size_bps })
        }
        Commands::EncodeLogCopyIntent {
            action,
            mint,
            amount_lamports,
            reference_sig_hex,
        } => {
            let action = match action.to_lowercase().as_str() {
                "buy" => CopyAction::Buy,
                "sell" => CopyAction::Sell,
                other => panic!("unknown action: {other}"),
            };
            print_instruction(TradeRegistryInstruction::LogCopyIntent {
                action,
                mint: pubkey(&mint),
                amount_lamports,
                reference_sig: hex_to_sig(&reference_sig_hex),
            });
        }
    }
}
