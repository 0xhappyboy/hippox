use crate::get_config;

/// Helper function to build identity introduction
pub(crate) fn get_identity_intro() -> String {
    let config = get_config();
    let identity = &config.identity_information;
    // Check if any identity field is set
    let has_name = identity.name.is_some();
    let has_role = identity.role.is_some();
    let has_personality = identity.personality.is_some();
    let has_species = identity.species.is_some();
    let has_age = identity.age.is_some();
    let has_sex = identity.sex.is_some();
    // No identity configured, use default
    if !has_name && !has_role && !has_personality && !has_species && !has_age && !has_sex {
        return "You are Hippox, a reliable AI runtime and skills orchestration engine with autonomous decision-making.".to_string();
    }
    // Build natural language introduction
    let mut intro = String::from("You are Hippox.");
    // Name
    if let Some(name) = &identity.name {
        intro.push_str(&format!(" Your name is {}.", name));
    }
    // Age and species combo
    if let Some(age) = &identity.age {
        if let Some(species) = &identity.species {
            intro.push_str(&format!(" You are a {} year old {}.", age, species));
        } else {
            intro.push_str(&format!(" You are {} years old.", age));
        }
    } else if let Some(species) = &identity.species {
        intro.push_str(&format!(" You are a {}.", species));
    }
    // Gender/sex
    if let Some(sex) = &identity.sex {
        let sex_word = match sex.to_lowercase().as_str() {
            "male" => "male",
            "female" => "female",
            _ => sex,
        };
        intro.push_str(&format!(" Your gender is {}.", sex_word));
    }
    // Role
    if let Some(role) = &identity.role {
        intro.push_str(&format!(" You work as a {}.", role));
    }
    // Personality
    if let Some(personality) = &identity.personality {
        intro.push_str(&format!(" You are known to be {}.", personality));
    }
    // Tone style
    if let Some(tone) = &identity.tone_style {
        intro.push_str(&format!(" You speak in a {} manner.", tone));
    }
    // Catchphrase
    if let Some(catchphrase) = &identity.catchphrase {
        intro.push_str(&format!(" Your catchphrase is \"{}\".", catchphrase));
    }
    // Append the core identity
    intro.push_str(" You are a reliable AI runtime and skills orchestration engine with autonomous decision-making.");
    intro
}
