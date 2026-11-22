import { Button, Popover } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { formatDistanceToNow } from "date-fns";
import { useMarkNotificationsRead, useNotifications } from "../../api/notifications";

export function NotificationsBell() {
  const { data: notifications = [], isError } = useNotifications();
  const markRead = useMarkNotificationsRead();
  const unread = notifications.filter((item) => !item.read).length;

  return (
    <Popover>
      <Button
        variant="ghost"
        isIconOnly
        aria-label={unread ? `Notifications, ${unread} unread` : "Notifications"}
        className="relative size-11 rounded-control text-ink-3 hover:bg-raised hover:text-ink-1"
      >
        <Icon icon="lucide:bell" width={20} height={20} />
        {unread > 0 && (
          <span className="absolute top-2 right-2 size-2 rounded-full bg-accent ring-2 ring-rail" />
        )}
      </Button>
      <Popover.Content placement="right bottom" className="w-80 rounded-panel border border-line-strong bg-input">
        <Popover.Dialog className="p-0">
          <header className="flex items-center justify-between border-b border-line px-4 py-3">
            <Popover.Heading className="font-mono text-[11px] tracking-widest text-ink-3 uppercase">
              Notifications
            </Popover.Heading>
            {unread > 0 && (
              <Button size="sm" variant="ghost" isPending={markRead.isPending} onPress={() => markRead.mutate()}>
                Mark all read
              </Button>
            )}
          </header>
          <ul className="max-h-96 overflow-y-auto">
            {notifications.length === 0 && (
              <li className="px-4 py-8 text-center text-sm text-ink-3">
                {isError ? "Notifications are unavailable." : "You're all caught up."}
              </li>
            )}
            {notifications.map((item) => (
              <li key={item.id} className="flex gap-3 border-b border-line px-4 py-3 last:border-b-0">
                <span className={item.read ? "mt-1.5 size-2 shrink-0" : "mt-1.5 size-2 shrink-0 rounded-full bg-accent"} />
                <div className="min-w-0 space-y-0.5">
                  <p className="text-sm font-medium text-ink-1">{item.title}</p>
                  {item.body && <p className="text-xs text-ink-3">{item.body}</p>}
                  <p className="font-mono text-[10px] text-ink-4 uppercase">
                    {formatDistanceToNow(new Date(item.created_at), { addSuffix: true })}
                  </p>
                </div>
              </li>
            ))}
          </ul>
        </Popover.Dialog>
      </Popover.Content>
    </Popover>
  );
}
