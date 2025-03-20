use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rn_core::cpu::{Assembler, Disassembler};
use std::fs;
use std::path::PathBuf;

/// NES Assembly tool for debugging and analysis
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assemble 6502 code and output binary or detailed information
    Assemble {
        /// The input file containing 6502 assembly code
        #[clap(value_parser)]
        input_file: PathBuf,

        /// The output file for assembled binary
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,

        /// Load address (in hex) for the assembled code
        #[clap(short, long, default_value = "8000")]
        address: String,

        /// Show verbose output including segment sizes and addresses
        #[clap(short, long)]
        verbose: bool,

        /// Also generate disassembly of the assembled code
        #[clap(short, long)]
        disassemble: bool,
    },
    
    /// Disassemble binary code to 6502 assembly
    Disassemble {
        /// The input binary file to disassemble
        #[clap(value_parser)]
        input_file: PathBuf,

        /// Start address for disassembly (in hex)
        #[clap(short, long, default_value = "8000")]
        address: String,

        /// Number of bytes to disassemble (default: entire file)
        #[clap(short, long)]
        length: Option<usize>,
        
        /// Show verbose output including hex bytes
        #[clap(short, long)]
        verbose: bool,
    },
    
    /// Show code from an ASM file with byte offsets and hex values
    Analyze {
        /// The input assembly file to analyze
        #[clap(value_parser)]
        input_file: PathBuf,
    }
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();

    // Process commands
    match cli.command {
        Commands::Assemble { 
            input_file, 
            output, 
            address, 
            verbose,
            disassemble
        } => {
            assemble_file(input_file, output, address, verbose, disassemble)?;
        },
        
        Commands::Disassemble { 
            input_file, 
            address, 
            length,
            verbose
        } => {
            disassemble_file(input_file, address, length, verbose)?;
        },
        
        Commands::Analyze { input_file } => {
            analyze_file(input_file)?;
        }
    }

    Ok(())
}

/// Assemble a file containing 6502 assembly code
fn assemble_file(
    input_file: PathBuf, 
    output: Option<PathBuf>, 
    address_str: String, 
    verbose: bool,
    disassemble: bool
) -> Result<()> {
    // Read input file
    let source_code = fs::read_to_string(&input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    // Parse the address
    let address = u16::from_str_radix(address_str.trim_start_matches("0x"), 16)
        .with_context(|| format!("Invalid address: {}", address_str))?;
    
    // Create assembler with proper address and NES segments
    let mut assembler = Assembler::new(address).with_nes_segments();
    
    // Assemble the code
    let segments = assembler.assemble_program(&source_code)
        .with_context(|| "Assembly failed")?;
    
    // Get the "STARTUP" segment by default, or the first segment if no STARTUP
    let primary_segment = if let Some(startup) = segments.get("STARTUP") {
        ("STARTUP", startup)
    } else if let Some((name, bytes)) = segments.iter().next() {
        (name.as_str(), bytes)
    } else {
        return Err(anyhow::anyhow!("No segments were assembled"));
    };
    
    // Print info
    if verbose {
        println!("Assembly successful!");
        println!("Segments:");
        for (name, bytes) in &segments {
            println!("  {}: {} bytes", name, bytes.len());
        }
        println!("\nPrimary segment: {} ({} bytes)", primary_segment.0, primary_segment.1.len());
    }
    
    // Write to output file if specified
    if let Some(output_path) = output {
        fs::write(&output_path, primary_segment.1)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
        println!("Binary written to: {}", output_path.display());
    }
    
    // Disassemble if requested
    if disassemble {
        println!("\nDisassembly:");
        let disassembler = Disassembler::new();
        
        // The bytes and starting address for disassembly
        let bytes = primary_segment.1;
        
        // Simple disassembly
        let disassembly = disassembler.disassemble_program(bytes, 0, bytes.len());
        
        // Print disassembly with proper formatting
        for (addr, bytes, instruction) in disassembly {
            let addr_with_base = addr as u16 + assembler.load_address;
            let bytes_hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            let bytes_str = bytes_hex.join(" ");
            println!("{:04X}: {:<8} {}", addr_with_base, bytes_str, instruction);
        }
    }
    
    Ok(())
}

/// Disassemble a binary file
fn disassemble_file(
    input_file: PathBuf, 
    address_str: String, 
    length: Option<usize>,
    verbose: bool
) -> Result<()> {
    // Read binary file
    let binary = fs::read(&input_file)
        .with_context(|| format!("Failed to read binary file: {}", input_file.display()))?;
    
    // Parse the address
    let address = u16::from_str_radix(address_str.trim_start_matches("0x"), 16)
        .with_context(|| format!("Invalid address: {}", address_str))?;
    
    // Limit the length if specified
    let binary = if let Some(len) = length {
        binary.iter().take(len).cloned().collect::<Vec<u8>>()
    } else {
        binary
    };
    
    // Create disassembler
    let disassembler = Disassembler::new();
    
    // Disassemble
    println!("Disassembly of {} at ${:04X}:", input_file.display(), address);
    
    let disassembly = disassembler.disassemble_program(&binary, 0, binary.len());
    
    // Format output based on verbosity
    for (addr, bytes, instruction) in disassembly {
        let addr_with_base = addr as u16 + address;
        if verbose {
            let bytes_hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
            let bytes_str = bytes_hex.join(" ");
            println!("{:04X}: {:<8} {}", addr_with_base, bytes_str, instruction);
        } else {
            println!("{:04X}: {}", addr_with_base, instruction);
        }
    }
    
    Ok(())
}

/// Analyze an assembly file by assembling it and showing detailed mapping
fn analyze_file(input_file: PathBuf) -> Result<()> {
    // Read input file
    let source_code = fs::read_to_string(&input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file.display()))?;
    
    // Split into lines for analysis
    let lines: Vec<&str> = source_code.lines().collect();
    
    // Create assembler with proper NES segments
    let mut assembler = Assembler::new(0x8000).with_nes_segments();
    
    // First, assemble the code to get all segments
    let segments = assembler.assemble_program(&source_code)
        .with_context(|| "Assembly failed")?;
    
    println!("Analysis of {}:", input_file.display());
    println!("Segments found:");
    for (name, bytes) in &segments {
        println!("  {}: {} bytes", name, bytes.len());
    }
    
    // Get the primary segment (STARTUP or first)
    let primary_segment = if let Some(startup) = segments.get("STARTUP") {
        ("STARTUP", startup)
    } else if let Some((name, bytes)) = segments.iter().next() {
        (name.as_str(), bytes)
    } else {
        return Err(anyhow::anyhow!("No segments were assembled"));
    };
    
    // Disassemble the primary segment
    let disassembler = Disassembler::new();
    let disassembly = disassembler.disassemble_program(primary_segment.1, 0, primary_segment.1.len());
    
    println!("\nDetailed mapping:");
    println!("Line | Address | Bytes       | Instruction");
    println!("-----------------------------------------");
    
    // TODO: In a full implementation, we'd track line numbers during assembly
    // For now, we'll just show the disassembly
    for (i, (addr, bytes, instruction)) in disassembly.iter().enumerate() {
        let addr_with_base = *addr as u16 + assembler.load_address;
        let bytes_hex: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
        let bytes_str = bytes_hex.join(" ");
        println!("{:4} | {:04X}    | {:<10} | {}", i + 1, addr_with_base, bytes_str, instruction);
    }
    
    Ok(())
}
