// Google Script that sets event colors based on emojis in the event title.

const mapping = {
  // Lecture
  "📚": CalendarApp.EventColor.MAUVE,
  // Lab
  "🧪": CalendarApp.EventColor.GREEN,
  // Exercise
  "🏋️": CalendarApp.EventColor.RED,
  // Seminar
  "📝": CalendarApp.EventColor.YELLOW,
  // PE
  "🏃": CalendarApp.EventColor.PALE_RED,
  // Languages
  "🗣️": CalendarApp.EventColor.CYAN,
  // Project
  "🛠️": CalendarApp.EventColor.GRAY,
};

function lerp(value, a1, b1, a2, b2) {
  const frac = (value - a1)/(b1 - a1);
  return a2 + frac * (b2 - a2);
}

function clamp(value, from, to) {
  if(value < from) return from;
  if(value > to) return to;

  return value;
}

function getPercent(value) {
  return Math.floor(value * 1000) / 10 + "%";
}

/**
 * Returns a string progress bar
 *
 * @param value Value, 0.0 to 1.0.
 */
function drawProgressBar(value) {
  const barLength = 20;
  const chars = "　▏▎▍▌▋▊▉█";
  value = clamp(value, 0, 1);

  let str = "";
  for(let i = 0; i < barLength; i++) {
    const start = i / barLength;
    const end = (i + 1) / barLength;

    const local = clamp(lerp(value, start, end, 0, 1), 0, 1);
    const idx = Math.floor(local * (chars.length - 1));

    str += chars[idx];
  }

  return str;
}

function setEventColorsByEmoji() {
  const props = PropertiesService.getScriptProperties();

  const calendarId = props.getProperty("CALENDAR_ID");

  if(!calendarId) {
    Logger.log("You have to set the calendar ID in script property CALENDAR_ID!");
    return;
  }

  const calendar = CalendarApp.getCalendarById(calendarId);

  if(!calendar) {
    Logger.log(`Calendar not found: ${calendarId}!`);
    return;
  }

  Logger.log(`Found calendar: ${calendar.getName()}`);

  const start = new Date();
  start.setFullYear(start.getFullYear() - 1);

  const end = new Date();
  end.setFullYear(end.getFullYear() + 1);

  const events = calendar.getEvents(start, end);

  Logger.log(`Got ${events.length} events!`);

  for(let i = 0; i < events.length; i++) {
    let event = events[i];
    let doneFrac = (i + 1) / events.length;
    let progress = "▕" + drawProgressBar(doneFrac) + "▏" + getPercent(doneFrac);
    let color = Object.entries(mapping).find(([emoji, color]) => event.getTitle().startsWith(emoji))?.[1];

    if(!color) {
      Logger.log(`${progress}\nColor not found for event: ${event.getTitle()} (${event.getId()} @ ${event.getStartTime()})`);
    } else {
      if(event.getColor() != color)
        event.setColor(color);
      Logger.log(`${progress}\nColor set for ${event.getTitle()} @ ${event.getStartTime()}!`);
    }
  }
}
