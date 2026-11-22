import { ToggleButton, cn } from "@heroui/react";
import type { Weekday } from "../../api/schedules";
import { WEEKDAYS } from "./model";

interface WeekdayPickerProps {
  value: Weekday[];
  onChange: (days: Weekday[]) => void;
}

export function WeekdayPicker({ value, onChange }: WeekdayPickerProps) {
  const toggle = (day: Weekday, on: boolean) =>
    onChange(on ? WEEKDAYS.filter((candidate) => candidate === day || value.includes(candidate)) : value.filter((candidate) => candidate !== day));

  return (
    <div className="flex flex-wrap gap-1.5">
      {WEEKDAYS.map((day) => {
        const selected = value.includes(day);
        return (
          <ToggleButton
            key={day}
            size="sm"
            isSelected={selected}
            onChange={(on: boolean) => toggle(day, on)}
            className={cn(
              "min-w-12 rounded-control border font-medium",
              selected ? "border-accent bg-accent-tint text-[#d9a8ff]" : "border-line bg-input text-ink-3 hover:text-ink-1",
            )}
          >
            {day}
          </ToggleButton>
        );
      })}
    </div>
  );
}
