import { Dropdown, type Selection } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useTranslation } from "react-i18next";
import { LANGUAGES } from "../../i18n";

export function LanguageChip() {
  const { t, i18n } = useTranslation();
  const current = i18n.resolvedLanguage ?? "en";

  const select = (keys: Selection) => {
    if (keys === "all") return;
    const [code] = keys;
    if (code) void i18n.changeLanguage(String(code));
  };

  return (
    <Dropdown>
      <Dropdown.Trigger
        aria-label={t("language.label")}
        className="inline-flex items-center gap-1.5 rounded-full border border-line-strong bg-input px-2.5 py-1 font-mono text-[11px] tracking-wider text-ink-1 uppercase hover:bg-raised"
      >
        <Icon icon="lucide:languages" width={14} height={14} />
        {current}
      </Dropdown.Trigger>
      <Dropdown.Popover placement="bottom start">
        <Dropdown.Menu
          aria-label={t("language.label")}
          selectionMode="single"
          disallowEmptySelection
          selectedKeys={[current]}
          onSelectionChange={select}
        >
          {LANGUAGES.map((language) => (
            <Dropdown.Item key={language.code} id={language.code} textValue={language.label}>
              {language.label}
              <Dropdown.ItemIndicator />
            </Dropdown.Item>
          ))}
        </Dropdown.Menu>
      </Dropdown.Popover>
    </Dropdown>
  );
}
