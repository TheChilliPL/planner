use crate::calendar::schedule::Schedule;
use crate::ical::SerializeToICal;
use clap::{Args, Parser, Subcommand, ValueEnum};
use eyre::{eyre, OptionExt};
use log::{debug, info, LevelFilter};
use qolor::color::BasicColor::Green;
use qolor::shorthands::Formattable;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use chrono::{Local, NaiveDate, Weekday};
use crate::time::timeext::TimeDeltaExt;

mod calendar;
mod ical;
mod time;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows the plan for the day.
    Show {
        #[command(flatten)]
        input: ScheduleInput,

        /// The date to show.
        ///
        /// Can be:
        /// "today", "tomorrow", "yesterday", "ereyesterday", "overmorrow",
        /// ISO date (yyyy-mm-dd),
        /// or W<week>D<day> (both 1-indexed).
        ///
        /// By default, uses today's date.
        #[arg(value_name = "DATE")]
        date: Option<String>,
    },
    /// Generates an iCal (.ics) file of the specified schedule.
    Generate {
        #[command(flatten)]
        input: ScheduleInput,
        /// Path at which the output .ics file will be saved.
        ///
        /// By default, uses the schedule .json path with .json replaced with `.ics`.
        #[arg(short, long, value_name = "OUTPUT_PATH")]
        output: Option<PathBuf>,
    },
}

#[derive(Args, Debug)]
pub struct ScheduleInput {
    /// Path to the schedule .json/.toml file.
    #[arg(value_name = "SCHEDULE_PATH", env = "PLANNER_SCHEDULE_PATH")]
    path: PathBuf,
    /// Format of the schedule. By default, detected based on file extension.
    #[arg(long, value_name = "SCHEDULE_FORMAT", env = "PLANNER_SCHEDULE_FORMAT")]
    format: Option<ScheduleFormat>,
}

impl ScheduleInput {
    pub fn effective_format(&self) -> eyre::Result<ScheduleFormat> {
        self.format.map(Ok).unwrap_or_else(|| ScheduleFormat::from_path(&self.path))
    }

    pub fn read(&self) -> eyre::Result<Schedule> {
        let format = self.effective_format()?;

        let data = read_to_string(&self.path)?;

        Ok(match format {
            ScheduleFormat::Json => serde_json::from_str(&data)?,
            ScheduleFormat::Toml => toml::from_str(&data)?,
            ScheduleFormat::Yaml => yaml_serde::from_str(&data)?,
        })
    }
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum ScheduleFormat {
    Json,
    Toml,
    Yaml,
}

impl ScheduleFormat {
    pub fn from_path(path: impl AsRef<Path>) -> eyre::Result<ScheduleFormat> {
        let ext = path.as_ref().extension().ok_or_eyre("no file extension")?
            .to_str().ok_or_eyre("extension is not valid UTF-8")?;
        Ok(Self::from_extension(ext).ok_or_eyre("unknown file extension format")?)
    }

    pub fn from_extension(ext: &str) -> Option<ScheduleFormat> {
        if ext.eq_ignore_ascii_case("json") {
            Some(ScheduleFormat::Json)
        } else if ext.eq_ignore_ascii_case("toml") {
            Some(ScheduleFormat::Toml)
        } else if ext.eq_ignore_ascii_case("yaml") {
            Some(ScheduleFormat::Yaml)
        } else {
            None
        }
    }
}

fn date_to_triple(date: NaiveDate, schedule: &Schedule)
    -> eyre::Result<(NonZero<usize>, Weekday, NaiveDate)> {
    let (week_no, weekday) = schedule.get_day(date)?;
    Ok((week_no, weekday, date))
}

fn parse_date(input: &str, schedule: &Schedule)
    -> eyre::Result<(NonZero<usize>, Weekday, NaiveDate)> {
    match input {
        "today" => {
            let date = Local::now().date_naive();

            date_to_triple(date, schedule)
        }
        "tomorrow" => {
            let date = Local::now().date_naive() + chrono::Duration::days(1);

            date_to_triple(date, schedule)
        }
        "yesterday" => {
            let date = Local::now().date_naive() - chrono::Duration::days(1);

            date_to_triple(date, schedule)
        }
        "ereyesterday" => {
            let date = Local::now().date_naive() - chrono::Duration::days(2);

            date_to_triple(date, schedule)
        }
        "overmorrow" => {
            let date = Local::now().date_naive() + chrono::Duration::days(2);

            date_to_triple(date, schedule)
        }
        _ if input.starts_with('W') && input.contains('D') => {
            let parts: Vec<&str> = input[1..].split('D').collect();

            if parts.len() != 2 {
                return Err(eyre!("Invalid W<week>D<day> format: {}", input));
            }

            let week_no: usize = parts[0].parse().map_err(|_| eyre!("Invalid week number in date: {}", input))?;
            let day_no: u32 = parts[1].parse().map_err(|_| eyre!("Invalid day number in date: {}", input))?;

            if week_no == 0 || day_no == 0 || day_no > 5 {
                return Err(eyre!("Week and day numbers must be positive and day must be <= 5: \
                {}", input));
            }

            let week_index = week_no - 1;
            let day_index = day_no - 1;

            if week_index >= schedule.weeks.len() {
                return Err(eyre!("Week number out of range: {}", input));
            }

            let date = schedule.weeks[week_index][day_index as usize];

            date_to_triple(date, schedule)
        }
        _ => {
            if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
                return date_to_triple(date, schedule);
            }

            Err(eyre!("Failed to parse date: {}", input))
        }
    }
}

fn main() -> eyre::Result<()> {
    // Initialize logger
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Cli::parse();

    match args.command {
        Commands::Show { input, date } => {
            let schedule: Schedule = input.read()?;

            let (week_no, weekday, date) = match date {
                Some(d) => parse_date(&d, &schedule),
                None => {
                    let date = Local::now().date_naive();
                    date_to_triple(date, &schedule)
                }
            }?;

            let is_today = date == Local::now().date_naive();

            println!(
                "{}",
                format!(" - {} | Week {}, {} - ", date, week_no, weekday)
                    .bg(Green)
                    .to_ansi()
            );

            let classes = schedule.get_classes_on(week_no, weekday).collect::<Vec<_>>();

            let time_now = Local::now().time();

            if is_today {
                if classes.is_empty() {
                    println!("{}", "You have no classes today!".dim().to_ansi());
                    return Ok(());
                }

                println!(
                    "{}",
                    format!("You have {} classes today:", classes.len())
                        .dim()
                        .to_ansi()
                );

                let classes_start_at = classes.first().unwrap().time.start;

                if classes_start_at > time_now {
                    let remaining = classes_start_at - time_now;
                    println!(
                        "{} until the first class!",
                        remaining.to_human_readable().bold().to_ansi()
                    );
                }
            }

            for class in &classes {
                let subject = schedule
                    .subjects
                    .get(&class.subject)
                    .ok_or_eyre("subject name not found")?;

                let mut first_line = format!("{} {}\n", class.class_type.to_emoji(), subject.get_short_or_name())
                    .fg(class.class_type.to_color());

                if is_today {
                    if class.time.end < time_now {
                        debug!("Class {} has already ended", subject.name);
                        first_line = first_line.strike();
                    } else if class.time.start <= time_now {
                        first_line = first_line.bold();
                    }
                }

                let text = first_line
                    + format!("    {}", class.time).dim();

                println!("{}", text.to_ansi());
            }

            if is_today {
                let class_end_at = classes.last().unwrap().time.end;

                if class_end_at > time_now {
                    let remaining = class_end_at - time_now;
                    println!(
                        "{} until the end!",
                        remaining.to_human_readable().bold().to_ansi()
                    );
                }
            }
        }
        Commands::Generate { input, output } => {
            let output = match output {
                Some(path) => path,
                None => input.path.with_extension("ics"),
            };

            debug!("Will be saving to {}", output.display());

            let schedule: Schedule = input.read()?;

            info!("Schedule: {:?}", schedule);

            let timezone = time::timezones::try_get_local_timezone()?;
            debug!("Using local timezone: {}", timezone);

            let ical = schedule.to_ical(&timezone)?;

            info!("Successfully generated events!");

            (&ical as &dyn SerializeToICal).serialize_to_ical_file(&output)?;

            info!("Successfully exported calendar to {}!", output.display());
        }
    }

    Ok(())
}
