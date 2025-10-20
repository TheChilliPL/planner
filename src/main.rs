use crate::calendar::schedule::Schedule;
use crate::ical::SerializeToICal;
use clap::{Parser, Subcommand};
use eyre::OptionExt;
use log::{debug, info, LevelFilter};
use qolor::color::BasicColor::Green;
use qolor::shorthands::Formattable;
use std::fs::File;
use std::path::PathBuf;
use chrono::Local;
use crate::time::timeext::TimeDeltaExt;

mod calendar;
mod ical;
mod time;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows the plan for the day.
    Show {
        /// Path to the schedule .json file
        #[arg(value_name = "SCHEDULE_PATH", env = "PLANNER_SCHEDULE_PATH")]
        path: PathBuf,
    },
    /// Generates an iCal (.ics) file of the specified schedule.
    Generate {
        /// Path to the schedule .json file
        #[arg(value_name = "SCHEDULE_PATH", env = "PLANNER_SCHEDULE_PATH")]
        path: PathBuf,
        /// Path at which the output .ics file will be saved.
        ///
        /// By default, uses the schedule .json path with .json replaced with `.ics`.
        #[arg(short, long, value_name = "OUTPUT_PATH")]
        output: Option<PathBuf>,
    },
}

fn main() -> eyre::Result<()> {
    // Initialize logger
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Show { path } => {
            let today = Local::now().date_naive();

            let schedule: Schedule = serde_json::from_reader(File::open(&path)?)?;

            let (week_no, weekday) = schedule.get_day(today)?;

            println!(
                "{}",
                format!(" - Week {}, {} - ", week_no, weekday)
                    .bg(Green)
                    .to_ansi()
            );

            let classes = schedule.get_classes_on(week_no, weekday).collect::<Vec<_>>();

            println!(
                "{}",
                format!("You have {} classes today:", classes.len())
                    .dim()
                    .to_ansi()
            );

            let time_now = Local::now().time();

            for class in &classes {
                let subject = schedule
                    .subjects
                    .get(&class.subject)
                    .ok_or_eyre("subject name not found")?;

                let mut first_line = format!("{} {}\n", class.class_type.to_emoji(), subject.get_short_or_name())
                    .fg(class.class_type.to_color());

                if class.time.end < time_now {
                    debug!("Class {} has already ended", subject.name);
                    first_line = first_line.strike();
                } else if class.time.start <= time_now {
                    first_line = first_line.bold();
                }

                let text = first_line
                    + format!("    {}", class.time).dim();

                println!("{}", text.to_ansi());
            }

            let class_end_at = classes.last().unwrap().time.end;

            if class_end_at > time_now {
                let remaining = class_end_at - time_now;
                println!(
                    "{} until the end!",
                    remaining.to_human_readable().bold().to_ansi()
                );
            }
        }
        Commands::Generate { path, output } => {
            let output = match output {
                Some(path) => path,
                None => path.with_extension("ics"),
            };

            debug!("Will be saving to {}", output.display());

            let schedule: Schedule = serde_json::from_reader(File::open(&path)?)?;

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
