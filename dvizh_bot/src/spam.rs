struct SpamRule {
    pattern: &'static str,
    score: i32,
}

static SPAM_SCORE: i32 = 10;

static SPAM_RULES: &[SpamRule] = &[
    SpamRule {
        pattern: "удалёнка",
        score: 3,
    },
    SpamRule {
        pattern: "удалённый",
        score: 3,
    },
    SpamRule {
        pattern: "заработок",
        score: 3,
    },
    SpamRule {
        pattern: "от 50$",
        score: 2,
    },
    SpamRule {
        pattern: "от 50 баксов",
        score: 2,
    },
    SpamRule {
        pattern: "от 100$",
        score: 3,
    },
    SpamRule {
        pattern: "от 100 баксов",
        score: 3,
    },
    SpamRule {
        pattern: "2-3 часа",
        score: 2,
    },
    SpamRule {
        pattern: "2-3 ч",
        score: 2,
    },
    SpamRule {
        pattern: "пиши в лс",
        score: 4,
    },
    SpamRule {
        pattern: "пишите в лс",
        score: 4,
    },
    SpamRule {
        pattern: "пиcaть в лc",
        score: 4,
    },
    SpamRule {
        pattern: "легально",
        score: 2,
    },
    SpamRule {
        pattern: "безопасно",
        score: 2,
    },
    SpamRule {
        pattern: "законно",
        score: 2,
    },
    SpamRule {
        pattern: "главное желание",
        score: 3,
    },
    SpamRule {
        pattern: "всё просто",
        score: 3,
    },
    SpamRule {
        pattern: "подработку",
        score: 4,
    },
    SpamRule {
        pattern: "подработка",
        score: 4,
    },
    SpamRule {
        pattern: "полностью легально",
        score: 2,
    },
    SpamRule {
        pattern: "проводится обучение",
        score: 2,
    },
];

pub fn is_spam_by_score(text: &str) -> bool {
    spam_score(text) >= SPAM_SCORE
}

fn spam_score(text: &str) -> i32 {
    let text_lower = text.to_lowercase();
    let mut total_score = 0;

    for rule in SPAM_RULES {
        if text_lower.contains(rule.pattern) {
            total_score += rule.score;
        }
    }

    total_score
}
