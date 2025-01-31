#[derive(Debug)]
pub enum CommandType {
    Start,
    Hello,
    Help,
    SetBirthdate,
    SetBirthdateFor,
    AddEvent,
    ListEvents,
    Meme,
    Astro,
    Luck,
    Patience,
    Joke,
    EightBall,
    Tease,
    Test,
}

pub fn command_str_to_type(t: &str) -> Option<CommandType> {
    match t.to_lowercase().as_str() {
        "start" => Some(CommandType::Start),
        "hello" => Some(CommandType::Hello),
        "help" => Some(CommandType::Help),
        "setbirthday" => Some(CommandType::SetBirthdate),
        "setbirthdayfor" => Some(CommandType::SetBirthdateFor),
        "addevent" => Some(CommandType::AddEvent),
        "listevents" => Some(CommandType::ListEvents),
        "meme" => Some(CommandType::Meme),
        "astro" => Some(CommandType::Astro),
        "luck" => Some(CommandType::Luck),
        "patience" => Some(CommandType::Patience),
        "joke" => Some(CommandType::Joke),
        "8ball" => Some(CommandType::EightBall),
        "tease" => Some(CommandType::Tease),
        "test" => Some(CommandType::Test),
        _ => None,
    }
}

pub fn parse_command_arguments(msg_text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in msg_text.chars() {
        match c {
            '"' | '“' | '”' | '[' | ']' => {
                in_quotes = !in_quotes;
            }
            ' ' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(c);
            }
        }
    }
    if !current_arg.is_empty() {
        args.push(current_arg.trim().to_string());
    }

    args
}
