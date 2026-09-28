use time::{Date, format_description::BorrowedFormatItem, macros::format_description};

const SHORT_DATE: &[BorrowedFormatItem<'_>] =
    format_description!("[day padding:none] [month repr:short] [year]");

pub(crate) fn short_date(date: Date) -> String {
    date.format(SHORT_DATE)
        .expect("the static date format is valid")
        .to_lowercase()
}
