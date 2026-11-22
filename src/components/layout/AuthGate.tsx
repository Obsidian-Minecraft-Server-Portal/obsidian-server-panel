import { useTranslation } from "react-i18next";
import { Button } from "@heroui/react";
import { Navigate, Outlet, useLocation } from "react-router";
import { ApiError } from "../../api/client";
import { useCurrentUser, useHealth } from "../../api/session";
import { AuroraBackground } from "../ui/AuroraBackground";
import { EmptyState } from "../ui/EmptyState";
import { Logo } from "../ui/Logo";
import { ProgressBar } from "../ui/ProgressBar";

const PUBLIC_PATHS = ["/login", "/reset-password", "/invite"];

function isPublicPath(pathname: string): boolean {
  return PUBLIC_PATHS.some((path) => pathname === path || pathname.startsWith(`${path}/`));
}

function Splash() {
  return (
    <AuroraBackground className="grid place-items-center">
      <div className="flex w-64 flex-col items-center gap-5">
        <Logo size={64} glow />
        <ProgressBar value={100} className="opacity-60" />
      </div>
    </AuroraBackground>
  );
}

export function AuthGate() {
  const { t } = useTranslation();
  const location = useLocation();
  const { pathname } = location;
  const health = useHealth();
  const setupRequired = health.data?.setup_required === true;
  const publicPath = isPublicPath(pathname);
  const needsSession = health.isSuccess && !setupRequired && (!publicPath || pathname === "/login");
  const me = useCurrentUser(needsSession);

  if (health.isPending) return <Splash />;

  if (health.isError) {
    return (
      <AuroraBackground className="grid place-items-center p-8">
        <div className="w-full max-w-lg">
          <EmptyState
            icon="lucide:plug-zap"
            title={t("gate.unreachableTitle")}
            description={t("gate.unreachableBody")}
            action={
              <Button variant="secondary" onPress={() => void health.refetch()}>
                {t("common.retry")}
              </Button>
            }
          />
        </div>
      </AuroraBackground>
    );
  }

  if (setupRequired) return pathname === "/setup" ? <Outlet /> : <Navigate to="/setup" replace />;
  if (pathname === "/setup") return <Navigate to="/login" replace />;
  if (publicPath) {
    const from = (location.state as { from?: string } | null)?.from;
    return pathname === "/login" && me.isSuccess ? <Navigate to={from ?? "/"} replace /> : <Outlet />;
  }

  if (me.isPending) return <Splash />;
  if (me.error instanceof ApiError && me.error.status === 401) {
    return <Navigate to="/login" replace state={{ from: pathname }} />;
  }
  if (me.isError) {
    return (
      <AuroraBackground className="grid place-items-center p-8">
        <div className="w-full max-w-lg">
          <EmptyState
            icon="lucide:triangle-alert"
            title={t("gate.sessionErrorTitle")}
            description={me.error.message}
            action={
              <Button variant="secondary" onPress={() => void me.refetch()}>
                {t("common.retry")}
              </Button>
            }
          />
        </div>
      </AuroraBackground>
    );
  }

  return <Outlet />;
}
