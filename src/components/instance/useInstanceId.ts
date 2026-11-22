import { useParams } from "react-router";

export function useInstanceId(): number {
  const { id } = useParams();
  return Number(id);
}
