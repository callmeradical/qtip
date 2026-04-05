use std::io::{self, Write};
use std::path::PathBuf;

const SKILL_CONTENT: &str = include_str!("skill_template.md");

#[derive(Debug)]
enum Agent {
    ClaudeCode,
    OpenAiCodex,
    GeminiCli,
}

impl Agent {
    fn skill_dir(&self) -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "Could not determine home directory".to_string())?;

        let base = match self {
            Agent::ClaudeCode => PathBuf::from(&home).join(".claude").join("skills"),
            Agent::OpenAiCodex => PathBuf::from(&home).join(".codex").join("skills"),
            Agent::GeminiCli => PathBuf::from(&home).join(".gemini").join("skills"),
        };

        Ok(base.join("qtip-scenarios"))
    }

    fn skill_filename(&self) -> &str {
        match self {
            Agent::ClaudeCode => "SKILL.md",
            Agent::OpenAiCodex => "SKILL.md",
            Agent::GeminiCli => "SKILL.md",
        }
    }

    fn display_name(&self) -> &str {
        match self {
            Agent::ClaudeCode => "Claude Code",
            Agent::OpenAiCodex => "OpenAI Codex",
            Agent::GeminiCli => "Gemini CLI",
        }
    }

    fn usage_hint(&self) -> &str {
        match self {
            Agent::ClaudeCode => {
                "Use /qtip-scenarios or ask Claude to generate scenarios from a PRD."
            }
            Agent::OpenAiCodex => "Ask Codex to generate qtip scenarios from a PRD or issue.",
            Agent::GeminiCli => "Ask Gemini to generate qtip scenarios from a PRD or issue.",
        }
    }
}

fn prompt_agent() -> Result<Agent, String> {
    println!("Which agent do you use?");
    println!();
    println!("  1) Claude Code");
    println!("  2) OpenAI Codex");
    println!("  3) Gemini CLI");
    println!();
    print!("Select [1-3]: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {e}"))?;

    match input.trim() {
        "1" => Ok(Agent::ClaudeCode),
        "2" => Ok(Agent::OpenAiCodex),
        "3" => Ok(Agent::GeminiCli),
        other => Err(format!("Invalid selection: {other}")),
    }
}

pub fn install_skill() -> Result<(), String> {
    let agent = prompt_agent()?;

    let skill_dir = agent.skill_dir()?;
    let skill_path = skill_dir.join(agent.skill_filename());

    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create {}: {e}", skill_dir.display()))?;

    std::fs::write(&skill_path, SKILL_CONTENT)
        .map_err(|e| format!("Failed to write {}: {e}", skill_path.display()))?;

    println!();
    println!(
        "Installed qtip-scenarios skill for {} at:",
        agent.display_name()
    );
    println!("  {}", skill_path.display());
    println!();
    println!("{}", agent.usage_hint());

    Ok(())
}
