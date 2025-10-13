use crate::ical::SerializeToICal;
use crate::calendar::schedule::{schedule_to_ical, Schedule};
use clap::{Parser, Subcommand};
use log::{debug, info, LevelFilter};
use std::fs::File;
use std::path::PathBuf;

mod calendar;
mod time;
mod ical;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates an iCal (.ics) file of the specified schedule.
    Generate {
        /// Path to the schedule .json file
        #[arg(value_name = "SCHEDULE_PATH")]
        path: PathBuf,
        /// Path at which the output .ics file will be saved.
        ///
        /// By default, uses the schedule .json path with .json replaced with `.ics`.
        #[arg(short, long, value_name = "OUTPUT_PATH")]
        output: Option<PathBuf>,
    }
}

fn main() -> eyre::Result<()> {
    // Initialize logger
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Generate { path, output } => {
            let output = match output {
                Some(path) => path,
                None => path.with_extension("ics")
            };

            debug!("Will be saving to {}", output.display());

            let schedule: Schedule = serde_json::from_reader(File::open(&path)?)?;

            info!("Schedule: {:?}", schedule);

            let timezone = crate::time::timezones::try_get_local_timezone()?;
            debug!("Using local timezone: {}", timezone);

            let ical = schedule_to_ical(&schedule, &timezone)?;

            info!("Successfully generated events!");

            (&ical as &dyn SerializeToICal).serialize_to_ical_file(&output)?;

            info!("Successfully exported calendar to {}!", output.display());
        }
    }

    // for (week_index, week) in weeks.iter().enumerate() {
    //     for (index, day) in week.iter().enumerate() {
    //         let weekday = Weekday::try_from(index as u8).unwrap();
    //         let real_weekday = day.weekday();
    //
    //         if weekday != real_weekday {
    //             println!("{} is {} instead of {}!", day, weekday, real_weekday);
    //         }
    //
    //         for class in &classes {
    //             if !class.week_mask.happens_in_week(week_index) {
    //                 continue;
    //             }
    //
    //             if class.day != weekday {
    //                 continue;
    //             }
    //
    //             let uid = format!(
    //                 "{}_{}_{}_{}_{}",
    //                 class.class_type.to_name(),
    //                 class.subject.replace(" ", "_"),
    //                 class.day,
    //                 week_index,
    //                 class.time.start.format("%H%M"),
    //             );
    //
    //             let now = Local::now();
    //             let start = day.and_time(class.time.start).and_local_timezone(timezone).unwrap();
    //             let end = day.and_time(class.time.end).and_local_timezone(timezone).unwrap();
    //
    //             let now_stamp = now.to_stamp();
    //             let start_stamp = start.to_stamp();
    //             let end_stamp = end.to_stamp();
    //
    //             let summary = format!(
    //                 "{} {}",
    //                 class.class_type.to_emoji(),
    //                 class.subject,
    //             );
    //
    //             let description = format!(
    //                 "{}\\n{}",
    //                 class.class_type.to_name(),
    //                 class.teacher,
    //             );
    //
    //             file.write(
    //                 format!(
    //                     concat!(
    //                         "BEGIN:VEVENT\n",
    //                         "UID:{}\n",
    //                         "DTSTAMP;TZID={}:{}\n",
    //                         "DTSTART;TZID={}:{}\n",
    //                         "DTEND;TZID={}:{}\n",
    //                         "SUMMARY:{}\n",
    //                         "LOCATION:{}\n",
    //                         "DESCRIPTION:{}\n",
    //                         "END:VEVENT\n",
    //                     ),
    //                     uid,
    //                     timezone.name(), now_stamp,
    //                     timezone.name(), start_stamp,
    //                     timezone.name(), end_stamp,
    //                     summary,
    //                     class.location,
    //                     description,
    //                 )
    //                 .as_bytes(),
    //             )
    //             .unwrap();
    //         }
    //     }
    // }
    //
    // file.write(b"END:VCALENDAR\n").unwrap();

    Ok(())
}
