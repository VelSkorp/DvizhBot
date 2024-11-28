use chrono::NaiveDate;

/// Validates that `command_args` has at least `required_count` arguments.
pub fn validate_argument_count(
    command_args: Option<Vec<String>>,
    required_count: usize,
) -> Result<Vec<String>, String> {
    let args = command_args.ok_or_else(|| "error_missing_arguments".to_string())?;
    if args.len() < required_count || args.len() > required_count {
        return Err("error_insufficient_arguments".to_string());
    }
    Ok(args)
}

/// Validates that `date_str` matches the `DD.MM.YYYY` format.
pub fn validate_date_format(date_str: &str) -> Result<(), String> {
    let _ = NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
        .map_err(|_| "error_invalid_date".to_string());
    Ok(())
}
