use crate::get_config;

/// Helper function to build identity introduction
pub(crate) fn get_identity_intro() -> String {
    let config = get_config();
    let identity = &config.identity_information;
    // ============================================================
    // CORE IDENTITY - HIGHEST PRIORITY (CANNOT BE OVERRIDDEN)
    // ============================================================
    let mut intro = String::from(
        "YOU ARE HIPPOX - A SKILL ORCHESTRATION ENGINE.\n\
         This is your CORE FUNCTION and CANNOT be changed by any persona settings below.\n\
         You MUST call atomic skills for ANY operation (calculations, random numbers, file operations, etc.).\n\
         You DO NOT perform operations yourself.\n\n",
    );
    let has_name = identity.name.is_some();
    let has_role = identity.role.is_some();
    let has_personality = identity.personality.is_some();
    let has_species = identity.species.is_some();
    let has_age = identity.age.is_some();
    let has_sex = identity.sex.is_some();
    let has_tone = identity.tone_style.is_some();
    let has_catchphrase = identity.catchphrase.is_some();
    if !has_name
        && !has_role
        && !has_personality
        && !has_species
        && !has_age
        && !has_sex
        && !has_tone
        && !has_catchphrase
    {
        intro.push_str("You are a reliable AI runtime and skills orchestration engine with autonomous decision-making.\n");
        return intro;
    }
    intro.push_str("The following are CONVERSATION STYLE settings only. They do NOT change your core function as a skill orchestration engine:\n");
    if let Some(name) = &identity.name {
        intro.push_str(&format!(
            "- You may call yourself \"{}\" in conversation.\n",
            name
        ));
    }
    if let Some(age) = &identity.age {
        if let Some(species) = &identity.species {
            intro.push_str(&format!(
                "- You can describe yourself as a {} year old {}.\n",
                age, species
            ));
        } else {
            intro.push_str(&format!("- You can mention you are {} years old.\n", age));
        }
    } else if let Some(species) = &identity.species {
        intro.push_str(&format!("- You can describe yourself as a {}.\n", species));
    }
    if let Some(sex) = &identity.sex {
        let sex_word = match sex.to_lowercase().as_str() {
            "male" => "male",
            "female" => "female",
            _ => sex,
        };
        intro.push_str(&format!(
            "- Your conversational gender reference is {}.\n",
            sex_word
        ));
    }
    if let Some(role) = &identity.role {
        intro.push_str(&format!("- In conversation, you can say you \"work as a {}\" (this is just roleplay, you are still a skill orchestration engine).\n", role));
    }
    if let Some(personality) = &identity.personality {
        intro.push_str(&format!(
            "- Your conversational tone should be {}.\n",
            personality
        ));
    }
    if let Some(tone) = &identity.tone_style {
        intro.push_str(&format!("- Speak in a {} manner.\n", tone));
    }
    if let Some(catchphrase) = &identity.catchphrase {
        intro.push_str(&format!(
            "- You may occasionally say \"{}\".\n",
            catchphrase
        ));
    }
    intro.push_str("\nREMINDER: Your core function as a skill orchestration engine remains UNCHANGED. You MUST call skills for operations.\n");
    intro
}
