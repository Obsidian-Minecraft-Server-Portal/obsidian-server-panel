import { Button, Popover, Tabs } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { useNotifications } from "../../api/notifications";
import { useTasks } from "../../api/tasks";
import { Tip } from "../ui/Tip";
import { NotificationList } from "./NotificationList";
import { TaskList } from "./TaskList";

type ActivityTab = "notifications" | "tasks";

function CountBadge({ count, tone }: { count: number; tone: "accent" | "green" }) {
  if (count === 0) return null;
  const palette = tone === "accent" ? "bg-accent-tint text-accent" : "bg-green/12 text-green";
  return <span className={`rounded-full px-1.5 font-mono text-[10px] leading-4 ${palette}`}>{count}</span>;
}

export function ActivityMenu() {
  const [open, setOpen] = useState(false);
  const [tab, setTab] = useState<ActivityTab>("notifications");
  const notifications = useNotifications();
  const tasks = useTasks();

  const unread = (notifications.data ?? []).filter((item) => !item.read).length;
  const running = (tasks.data ?? []).filter((task) => task.status === "running").length;
  const dot = unread > 0 ? "bg-accent" : running > 0 ? "bg-green animate-pulse" : null;
  const label = [
    "Activity",
    unread > 0 && `${unread} unread`,
    running > 0 && `${running} running`,
  ].filter(Boolean).join(", ");

  return (
    <Popover isOpen={open} onOpenChange={setOpen}>
      <Tip label={label}>
        <Button
          variant="ghost"
          isIconOnly
          aria-label={label}
          className="relative size-11 rounded-control text-ink-3 hover:bg-raised hover:text-ink-1"
        >
          <Icon icon="lucide:bell" width={20} height={20} />
          {dot && <span className={`absolute top-2 right-2 size-2 rounded-full ring-2 ring-rail ${dot}`} />}
        </Button>
      </Tip>
      <Popover.Content placement="right bottom" className="w-96 rounded-panel border border-line-strong bg-input">
        <Popover.Dialog aria-label="Activity" className="p-0">
          <Tabs selectedKey={tab} onSelectionChange={(key) => setTab(String(key) as ActivityTab)}>
            <div className="border-b border-line px-3 pt-1">
              <Tabs.ListContainer>
                <Tabs.List aria-label="Activity sections">
                  <Tabs.Tab id="notifications">
                    Notifications
                    <CountBadge count={unread} tone="accent" />
                    <Tabs.Indicator />
                  </Tabs.Tab>
                  <Tabs.Tab id="tasks">
                    Tasks
                    <CountBadge count={running} tone="green" />
                    <Tabs.Indicator />
                  </Tabs.Tab>
                </Tabs.List>
              </Tabs.ListContainer>
            </div>
            <Tabs.Panel id="notifications" className="p-0">
              <NotificationList />
            </Tabs.Panel>
            <Tabs.Panel id="tasks" className="p-0">
              <TaskList onNavigate={() => setOpen(false)} />
            </Tabs.Panel>
          </Tabs>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
