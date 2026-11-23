import { Icon } from "@iconify-icon/react";
import { format } from "date-fns";
import type { ReactNode } from "react";
import type { ProjectDetail, ProjectLinks, SideSupport } from "../../../api/content";
import { Chip } from "../../ui/Chip";
import { EmptyState } from "../../ui/EmptyState";
import { Panel } from "../../ui/Panel";
import type { Tone } from "../../ui/tone";
import { SIDE_LABELS, httpUrl } from "../meta";

const CHIP_LIMIT = 12;

interface ProjectOverviewProps {
  detail: ProjectDetail;
}

export function ProjectOverview({ detail }: ProjectOverviewProps) {
  return (
    <div className="grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_320px]">
      <Description html={detail.body_html} />
      <aside className="space-y-5">
        <DetailsPanel detail={detail} />
        <LinksPanel links={detail.links} />
      </aside>
    </div>
  );
}

function Description({ html }: { html: string }) {
  if (html.trim() === "") {
    return (
      <EmptyState icon="lucide:file-text" title="No description" description="The author has not written one yet." />
    );
  }
  // Server-sanitized with ammonia (crates/obsidian-content/src/render.rs), so it is safe to inject.
  return <article className="rendered-html min-w-0" dangerouslySetInnerHTML={{ __html: html }} />;
}

function DetailsPanel({ detail }: { detail: ProjectDetail }) {
  const { project } = detail;
  const sides = [
    { label: "Client", side: project.client_side },
    { label: "Server", side: project.server_side },
  ].filter((entry): entry is { label: string; side: SideSupport } => entry.side !== null);

  return (
    <Panel title="Details">
      <div className="space-y-5">
        <dl className="grid grid-cols-[auto_minmax(0,1fr)] items-baseline gap-x-4 gap-y-3 text-sm">
          <Row label="License">{detail.license ?? "—"}</Row>
          <Row label="Published">{formatDate(detail.published_at)}</Row>
          <Row label="Updated">{formatDate(detail.updated_at)}</Row>
          {detail.followers !== null && <Row label="Followers">{detail.followers.toLocaleString()}</Row>}
          <Row label="Environment">
            {sides.length > 0 ? (
              <div className="flex flex-wrap gap-1.5">
                {sides.map(({ label, side }) => (
                  <Chip key={label} tone={side === "unsupported" ? "rose" : "neutral"}>
                    {label} · {SIDE_LABELS[side]}
                  </Chip>
                ))}
              </div>
            ) : (
              "—"
            )}
          </Row>
        </dl>
        <ChipGroup label="Loaders" items={detail.loaders} tone="secondary" />
        <ChipGroup label="Minecraft" items={detail.game_versions} tone="secondary" />
        <ChipGroup label="Categories" items={detail.categories} />
      </div>
    </Panel>
  );
}

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <>
      <dt className="text-ink-3">{label}</dt>
      <dd className="min-w-0 break-words text-ink-1">{children}</dd>
    </>
  );
}

function ChipGroup({ label, items, tone = "neutral" }: { label: string; items: string[]; tone?: Tone }) {
  if (items.length === 0) return null;
  const shown = items.slice(0, CHIP_LIMIT);
  return (
    <div className="space-y-2">
      <p className="font-mono text-[11px] tracking-wider text-ink-3 uppercase">{label}</p>
      <div className="flex flex-wrap gap-1.5">
        {shown.map((item) => (
          <Chip key={item} tone={tone}>
            {item}
          </Chip>
        ))}
        {items.length > shown.length && <Chip>+{items.length - shown.length}</Chip>}
      </div>
    </div>
  );
}

interface LinkItem {
  label: string;
  icon: string;
  href: string;
}

function linkItems(links: ProjectLinks): LinkItem[] {
  const candidates: [label: string, icon: string, url: string | null][] = [
    ["Website", "lucide:globe", links.website],
    ["Source", "lucide:code", links.source],
    ["Issues", "lucide:bug", links.issues],
    ["Wiki", "lucide:book-open", links.wiki],
    ["Discord", "lucide:message-square", links.discord],
    ...links.donate.map((url): [string, string, string | null] => ["Donate", "lucide:heart", url]),
  ];
  return candidates.flatMap(([label, icon, url]) => {
    const href = httpUrl(url);
    return href ? [{ label, icon, href }] : [];
  });
}

function LinksPanel({ links }: { links: ProjectLinks }) {
  const items = linkItems(links);
  if (items.length === 0) return null;
  return (
    <Panel title="Links">
      <ul className="-mx-2 space-y-0.5">
        {items.map((item) => (
          <li key={`${item.label}:${item.href}`}>
            <a
              href={item.href}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2.5 rounded-control px-2 py-1.5 text-sm text-ink-1 transition-colors hover:bg-raised"
            >
              <Icon icon={item.icon} width={14} height={14} className="text-ink-3" />
              {item.label}
            </a>
          </li>
        ))}
      </ul>
    </Panel>
  );
}

function formatDate(value: string | null): string {
  return value ? format(new Date(value), "d MMM yyyy") : "—";
}
