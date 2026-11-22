import { Button } from "@heroui/react";
import { formatDistanceToNow } from "date-fns";
import { useMarkNotificationsRead, useNotifications } from "../../api/notifications";

export function NotificationList() {
  const { data: notifications = [], isError } = useNotifications();
  const markRead = useMarkNotificationsRead();
  const unread = notifications.filter((item) => !item.read).length;

  return (
    <>
      {unread > 0 && (
        <div className="flex justify-end border-b border-line px-3 py-2">
          <Button size="sm" variant="ghost" isPending={markRead.isPending} onPress={() => markRead.mutate()}>
            Mark all read
          </Button>
        </div>
      )}
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
    </>
  );
}
