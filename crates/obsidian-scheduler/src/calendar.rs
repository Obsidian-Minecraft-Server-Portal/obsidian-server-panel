use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;

use crate::model::{Action, IntervalUnit, Schedule, Step, Trigger};

const WEEK_DAYS: u32 = 8;
const MONTH_SEARCH: u32 = 48;

pub fn describe(schedule: &Schedule) -> String {
	let when = describe_trigger(&schedule.trigger);
	if schedule.steps.is_empty() {
		return when;
	}
	let steps = schedule.steps.iter().map(describe_step).collect::<Vec<_>>().join(", then ");
	format!("{when}: {steps}")
}

pub fn next_runs(schedule: &Schedule, tz: Tz, count: usize) -> Vec<DateTime<Tz>> {
	next_runs_after(schedule, tz, Utc::now(), count)
}

pub fn next_runs_after(schedule: &Schedule, tz: Tz, after: DateTime<Utc>, count: usize) -> Vec<DateTime<Tz>> {
	let mut runs = Vec::with_capacity(count);
	let mut cursor = after;
	while runs.len() < count {
		let Some(next) = next_after(&schedule.trigger, tz, cursor) else { break };
		cursor = next.with_timezone(&Utc);
		runs.push(next);
	}
	runs
}

pub fn next_after(trigger: &Trigger, tz: Tz, after: DateTime<Utc>) -> Option<DateTime<Tz>> {
	match trigger {
		Trigger::Interval { every, unit } => {
			let minutes = match unit {
				IntervalUnit::Minutes => i64::from(*every),
				IntervalUnit::Hours => i64::from(*every) * 60,
			};
			let next = after.checked_add_signed(chrono::TimeDelta::try_minutes(minutes)?)?;
			Some(next.with_timezone(&tz))
		}
		Trigger::Daily { time } => calendar_next(tz, after, WEEK_DAYS, |_| true, *time),
		Trigger::Weekly { days, time } => {
			calendar_next(tz, after, WEEK_DAYS, |date| days.contains(&date.weekday()), *time)
		}
		Trigger::Monthly { day, time } => monthly_next(tz, after, u32::from(*day), *time),
		Trigger::Once { at } => tz
			.from_local_datetime(at)
			.earliest()
			.filter(|run| run.with_timezone(&Utc) > after),
		Trigger::OnStart | Trigger::OnStop | Trigger::OnCrash | Trigger::EmptyFor { .. } | Trigger::PlayerJoined => {
			None
		}
	}
}

fn at(tz: Tz, date: NaiveDate, time: NaiveTime) -> Option<DateTime<Tz>> {
	tz.from_local_datetime(&date.and_time(time)).earliest()
}

fn calendar_next(
	tz: Tz,
	after: DateTime<Utc>,
	horizon_days: u32,
	matches: impl Fn(NaiveDate) -> bool,
	time: NaiveTime,
) -> Option<DateTime<Tz>> {
	let mut date = after.with_timezone(&tz).date_naive();
	for _ in 0..horizon_days {
		if matches(date)
			&& let Some(run) = at(tz, date, time).filter(|run| run.with_timezone(&Utc) > after)
		{
			return Some(run);
		}
		date = date.succ_opt()?;
	}
	None
}

fn monthly_next(tz: Tz, after: DateTime<Utc>, day: u32, time: NaiveTime) -> Option<DateTime<Tz>> {
	let local = after.with_timezone(&tz).date_naive();
	let mut first = NaiveDate::from_ymd_opt(local.year(), local.month(), 1)?;
	for _ in 0..MONTH_SEARCH {
		if let Some(run) = first
			.with_day(day)
			.and_then(|date| at(tz, date, time))
			.filter(|run| run.with_timezone(&Utc) > after)
		{
			return Some(run);
		}
		first = first.checked_add_months(Months::new(1))?;
	}
	None
}

fn describe_trigger(trigger: &Trigger) -> String {
	match trigger {
		Trigger::Interval { every, unit } => {
			let unit = match unit {
				IntervalUnit::Minutes => "minute",
				IntervalUnit::Hours => "hour",
			};
			if *every == 1 { format!("Every {unit}") } else { format!("Every {every} {unit}s") }
		}
		Trigger::Daily { time } => format!("Every day at {}", clock(time)),
		Trigger::Weekly { days, time } => format!("Every {} at {}", join_days(days), clock(time)),
		Trigger::Monthly { day, time } => format!("On the {} of every month at {}", ordinal(*day), clock(time)),
		Trigger::Once { at } => format!("Once on {} at {}", at.format("%b %-d, %Y"), clock(&at.time())),
		Trigger::OnStart => "When the server starts".into(),
		Trigger::OnStop => "When the server stops".into(),
		Trigger::OnCrash => "When the server crashes".into(),
		Trigger::EmptyFor { minutes } => {
			let plural = if *minutes == 1 { "" } else { "s" };
			format!("When the server has been empty for {minutes} minute{plural}")
		}
		Trigger::PlayerJoined => "When a player joins".into(),
	}
}

pub(crate) fn describe_step(step: &Step) -> String {
	let mut text = match &step.action {
		Action::Backup => "back up".to_owned(),
		Action::Restart { countdown_secs } if countdown_secs.is_empty() => "restart".to_owned(),
		Action::Restart { countdown_secs } => {
			let mut stages = countdown_secs.clone();
			stages.sort_unstable_by(|a, b| b.cmp(a));
			let warnings = stages.iter().map(|secs| duration_label(*secs)).collect::<Vec<_>>().join(", ");
			format!("restart with warnings at {warnings}")
		}
		Action::Command { line } => format!("run `{line}`"),
		Action::Start => "start".to_owned(),
		Action::Stop => "stop".to_owned(),
		Action::UpdateContent => "update content".to_owned(),
	};
	if step.delay_secs > 0 {
		text.push_str(&format!(" after {}s", step.delay_secs));
	}
	if step.only_if_running {
		text.push_str(" if running");
	}
	text
}

pub(crate) fn duration_label(secs: u32) -> String {
	if secs >= 60 && secs.is_multiple_of(60) { format!("{}m", secs / 60) } else { format!("{secs}s") }
}

fn clock(time: &NaiveTime) -> String {
	time.format("%-I:%M %p").to_string()
}

fn join_days(days: &[Weekday]) -> String {
	let mut days = days.to_vec();
	days.sort_by_key(Weekday::num_days_from_monday);
	days.dedup();
	let names: Vec<String> = days.iter().map(ToString::to_string).collect();
	match names.split_last() {
		Some((last, [])) => last.clone(),
		Some((last, rest)) => format!("{} & {last}", rest.join(", ")),
		None => String::new(),
	}
}

fn ordinal(day: u8) -> String {
	let suffix = match (day % 100, day % 10) {
		(11..=13, _) => "th",
		(_, 1) => "st",
		(_, 2) => "nd",
		(_, 3) => "rd",
		_ => "th",
	};
	format!("{day}{suffix}")
}
