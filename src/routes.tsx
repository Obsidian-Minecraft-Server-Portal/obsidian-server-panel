import type { ComponentType } from "react";
import { createBrowserRouter, Navigate } from "react-router";
import { AppShell } from "./components/layout/AppShell";
import { AuthGate } from "./components/layout/AuthGate";

function page<M>(load: () => Promise<M>, name: keyof M) {
  return async () => ({ Component: (await load())[name] as ComponentType });
}

export const router = createBrowserRouter([
  {
    element: <AuthGate />,
    children: [
      { path: "/setup", lazy: page(() => import("./pages/setup/SetupPage"), "SetupPage") },
      { path: "/login", lazy: page(() => import("./pages/login/LoginPage"), "LoginPage") },
      { path: "/reset-password", lazy: page(() => import("./pages/reset/ResetPasswordPage"), "ResetPasswordPage") },
      { path: "/invite/:token", lazy: page(() => import("./pages/invite/InvitePage"), "InvitePage") },
      {
        element: <AppShell />,
        children: [
          { index: true, lazy: page(() => import("./pages/dashboard/DashboardPage"), "DashboardPage") },
          { path: "instances", element: <Navigate to="/" replace /> },
          {
            path: "instances/new",
            lazy: page(() => import("./pages/instances/CreateInstancePage"), "CreateInstancePage"),
          },
          { path: "instances/:id/:tab?", lazy: page(() => import("./pages/instance/InstancePage"), "InstancePage") },
          { path: "discover", lazy: page(() => import("./pages/discover/DiscoverPage"), "DiscoverPage") },
          { path: "users", element: <Navigate to="/settings/users" replace /> },
          { path: "account", element: <Navigate to="/settings/account" replace /> },
          { path: "settings/:section?", lazy: page(() => import("./pages/settings/SettingsPage"), "SettingsPage") },
        ],
      },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);
