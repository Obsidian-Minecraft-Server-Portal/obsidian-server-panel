import { Navigate, Outlet } from "react-router";
import { useCurrentUser } from "../../api/session";

export function AdminRoute() {
  const { data: user } = useCurrentUser();
  return user?.is_admin ? <Outlet /> : <Navigate to="/" replace />;
}
