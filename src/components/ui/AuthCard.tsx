import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useHealth } from "../../api/session";
import { AuroraBackground } from "./AuroraBackground";
import { GlassCard } from "./GlassCard";
import { LanguageChip } from "./LanguageChip";
import { Logo } from "./Logo";
import { StatusDot } from "./StatusDot";

interface AuthCardProps {
  headline: ReactNode;
  subtitle?: ReactNode;
  children?: ReactNode;
  footer?: ReactNode;
}

export function AuthCard({ headline, subtitle, children, footer }: AuthCardProps) {
  const { t } = useTranslation();
  const { data: health } = useHealth();
  const version = health?.version ? `v${health.version}` : "";

  return (
    <AuroraBackground className="flex flex-col">
      <div className="absolute top-4 left-4 z-10">
        <LanguageChip />
      </div>
      <main className="grid flex-1 place-items-center px-4 py-12">
        <GlassCard glow className="w-full max-w-md p-8">
          <div className="space-y-6">
            <div className="flex items-center gap-4">
              <span className="grid size-14 place-items-center rounded-control border border-line-strong bg-input shadow-[0_0_24px_-4px_var(--accent-glow)]">
                <Logo size={34} />
              </span>
              <div>
                <p className="font-display text-2xl font-bold text-ink-0">Obsidian</p>
                <p className="font-mono text-[11px] tracking-widest text-ink-3 uppercase">
                  {t("auth.serverPanel")}
                  {version && <span className="normal-case"> · {version}</span>}
                </p>
              </div>
            </div>
            <div className="space-y-2">
              <h1 className="font-display text-3xl font-bold text-ink-0">{headline}</h1>
              {subtitle && <p className="text-sm text-ink-2">{subtitle}</p>}
            </div>
            {children}
            <div className="flex items-center justify-between gap-4 border-t border-line pt-4 text-xs text-ink-4">
              <span>{t("auth.notAffiliated")}</span>
              {footer}
            </div>
          </div>
        </GlassCard>
      </main>
      <footer className="flex items-center justify-between gap-4 px-6 pb-5 font-mono text-[11px] tracking-widest text-ink-3 uppercase">
        <span className="flex items-center gap-2.5">
          <StatusDot tone="accent" pulse />
          {t("auth.panelOnline", { count: health?.running_instances ?? 0 })}
        </span>
        <span className="normal-case">{version}</span>
      </footer>
    </AuroraBackground>
  );
}
