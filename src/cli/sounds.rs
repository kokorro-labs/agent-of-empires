//! `agent-of-empires sounds` subcommands implementation

use anyhow::Result;
use clap::Subcommand;

use crate::sound;

#[derive(Subcommand)]
pub enum SoundsCommands {
    /// Install bundled sound effects
    Install,

    /// List currently installed sounds
    #[command(alias = "ls")]
    List,

    /// Test a sound by playing it
    Test {
        /// Sound file name (without extension)
        name: String,
    },
}

pub async fn run(command: SoundsCommands) -> Result<()> {
    match command {
        SoundsCommands::Install => install_bundled().await,
        SoundsCommands::List => list_sounds().await,
        SoundsCommands::Test { name } => test_sound(&name).await,
    }
}

async fn install_bundled() -> Result<()> {
    sound::install_bundled_sounds()?;

    if let Some(sounds_dir) = sound::get_sounds_dir() {
        println!("✓ Installed bundled CC0 sounds to:");
        println!("  {}\n", sounds_dir.display());

        let sounds = sound::list_available_sounds();
        println!("📂 Installed {} sounds:", sounds.len());
        for sound_name in sounds {
            println!("  • {}", sound_name);
        }

        println!("\n💡 Next steps:");
        println!("  1. Launch the TUI: aoe");
        println!("  2. Press 's' to open Settings");
        println!("  3. Navigate to Sound category");
        println!("  4. Enable sounds and configure transitions");

        println!("\n🎮 Want Age of Empires II sounds instead?");
        println!("   If you own AoE II, copy the taunt .wav files from:");
        println!("   • (AoE II dir)/resources/_common/sound/taunt/");
        println!("   • Or: (AoE II dir)/Sound/taunt/");
        println!("   To: {}", sounds_dir.display());
        println!("\n   Then configure which sounds to use in Settings!");
    }

    Ok(())
}

async fn list_sounds() -> Result<()> {
    let sounds = sound::list_available_sounds();

    if sounds.is_empty() {
        println!("No sounds installed yet.");
        println!("\nRun 'aoe sounds install --from interactive' to get started.");
        return Ok(());
    }

    println!("📂 Installed sounds:");
    for sound_name in &sounds {
        println!("  • {}", sound_name);
    }
    println!("\nTotal: {} sounds", sounds.len());

    if let Some(sounds_dir) = sound::get_sounds_dir() {
        println!("\nLocation: {}", sounds_dir.display());
    }

    println!("\n💡 Test a sound: aoe sounds test <name>");

    Ok(())
}

async fn test_sound(name: &str) -> Result<()> {
    let sounds = sound::list_available_sounds();

    if !sounds.contains(&name.to_string()) {
        println!("❌ Sound '{}' not found.", name);
        println!("\n📂 Available sounds:");
        for sound_name in sounds {
            println!("  • {}", sound_name);
        }
        return Ok(());
    }

    println!("🔊 Playing '{}'...", name);
    sound::play_sound(name);
    println!("   (If you don't hear anything, check your audio settings)");

    Ok(())
}
