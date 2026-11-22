import { Icon } from "@iconify-icon/react";
import { Button, cn } from "@heroui/react";
import { useMemo, useState, type FormEvent, type KeyboardEvent } from "react";
import { suggestCommands } from "./commands";

const MAX_HISTORY = 100;

interface CommandInputProps {
  disabled: boolean;
  onSubmit: (line: string) => void;
}

export function CommandInput({ disabled, onSubmit }: CommandInputProps) {
  const [value, setValue] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState<number | null>(null);
  const [highlight, setHighlight] = useState(0);
  const suggestions = useMemo(() => suggestCommands(value), [value]);
  const activeSuggestion = suggestions[Math.min(highlight, suggestions.length - 1)];

  const changeValue = (next: string) => {
    setValue(next);
    setHighlight(0);
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    const line = value.trim();
    if (!line || disabled) return;
    onSubmit(line);
    setHistory((past) => [line, ...past.filter((entry) => entry !== line)].slice(0, MAX_HISTORY));
    setHistoryIndex(null);
    changeValue("");
  };

  const recall = (direction: 1 | -1) => {
    if (history.length === 0) return;
    const next = Math.min(Math.max((historyIndex ?? -1) + direction, -1), history.length - 1);
    setHistoryIndex(next === -1 ? null : next);
    changeValue(next === -1 ? "" : history[next]);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Tab" && activeSuggestion) {
      event.preventDefault();
      changeValue(`${activeSuggestion} `);
    } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      const direction = event.key === "ArrowUp" ? -1 : 1;
      if (suggestions.length > 0) {
        setHighlight((index) => (index + direction + suggestions.length) % suggestions.length);
      } else {
        recall(direction === -1 ? 1 : -1);
      }
    }
  };

  return (
    <form onSubmit={submit} className="relative">
      {suggestions.length > 0 && !disabled && (
        <ul
          role="listbox"
          aria-label="Command suggestions"
          className="absolute bottom-full left-0 mb-2 w-full overflow-hidden rounded-panel border border-line-strong bg-raised shadow-[0_20px_50px_-20px_rgba(0,0,0,0.8)]"
        >
          {suggestions.map((command) => (
            <li
              key={command}
              role="option"
              aria-selected={command === activeSuggestion}
              className={cn(
                "flex items-center justify-between px-4 py-2 font-mono text-sm",
                command === activeSuggestion ? "bg-raised text-ink-0" : "text-ink-2",
              )}
            >
              {command}
              {command === activeSuggestion && <span className="font-mono text-[10px] text-ink-3">TAB</span>}
            </li>
          ))}
        </ul>
      )}
      <div className="flex items-center gap-3 rounded-control border border-line bg-input px-4 focus-within:border-accent">
        <input
          value={value}
          disabled={disabled}
          onChange={(event) => changeValue(event.target.value)}
          onKeyDown={onKeyDown}
          aria-label="Server command"
          placeholder={disabled ? "Start the server to send commands" : "Type a command, Tab to complete"}
          autoComplete="off"
          spellCheck={false}
          className="h-11 min-w-0 flex-1 bg-transparent font-mono text-sm text-ink-1 outline-none placeholder:text-ink-4 disabled:cursor-not-allowed"
        />
        <Button
          type="submit"
          isIconOnly
          isDisabled={disabled || !value.trim()}
          aria-label="Send command"
          className="size-8 min-w-8 rounded-control bg-accent text-accent-ink transition disabled:opacity-40"
        >
          <Icon icon="lucide:corner-down-left" width={16} height={16} />
        </Button>
      </div>
    </form>
  );
}
