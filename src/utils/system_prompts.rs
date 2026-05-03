use serde::Deserialize;
use std::{env::current_dir, path::PathBuf};

#[derive(Debug, Deserialize)]
struct SkillDef {
    name: String,
    description: String,
    #[serde(rename = "last-updated")]
    last_updated: Option<String>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<String>,
    compatibility: Option<String>,
}

fn get_skills_directory_path() -> String {
    let cwd = current_dir().expect("Couldnt get CWD");
    let cwd_path = cwd.to_str().expect("Path is invalid");
    let mut path_to_skills = PathBuf::from(cwd_path);
    path_to_skills.push(".skills");
    String::from(path_to_skills.to_str().expect("Skills path invalid"))
}
fn parse_skill_file(content: String) -> Option<SkillDef> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    let yaml_part = parts.get(1)?;
    let yaml = serde_yaml::from_str(yaml_part);

    return yaml.ok();
}
fn get_skills_definitions() -> Result<Vec<SkillDef>, std::io::Error> {
    let skills_path = get_skills_directory_path();

    let skills_files = std::fs::read_dir(skills_path)
        .expect(".skills directory not found")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                let mut final_path = entry.path().clone();
                final_path.push("SKILL.md");
                Some(final_path)
            } else {
                None
            }
        });

    let skills_files_content = skills_files.map(|path| std::fs::read_to_string(path));

    let skills = skills_files_content
        .map(|content| {
            let c = content?;
            parse_skill_file(c)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Parse Error"))
        })
        .collect::<Result<Vec<SkillDef>, std::io::Error>>()?;
    Ok(skills)
}

pub fn get_system_prompt() -> String {
    let skills = get_skills_definitions().unwrap();
    let skills_str = skills.iter().fold(String::new(), |mut acc, skill| {
        if !acc.is_empty() {
            acc.push('\n');
        }
        acc.push_str(&format!("{} - {}", skill.name, skill.description));

        return acc;
    });
    let system_prompt = format!(
        "The following skills are available for you to choose from using the ReadSkill tool:
        {}
        
        IMPORTANT: When asking to read references in the context of a skill, use the ReadSkill tool with both arguments ! DO NOT attempt to read reference directly using Read tool
        Example: When wanting to read this reference [deploying.md](references/deploying.md) in the firebase-hosting-basics skill
        call the ReadSkill tool with the following arguments {{ \"skill_name\": \"firebase-hosting-basics\" \"reference_path\": \"references/deploying.md\" }}
        ",
        skills_str
    );

    return system_prompt;
}
