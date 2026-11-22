import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type ApiError } from "../../../api/client";
import { putInstance, type InstanceView } from "../../../api/instances";
import { uploadFile } from "../../files/uploadFile";
import { createBody, isCustom, settingsPatch, type CreateDraft } from "./draft";

const CUSTOM_JAR_PATH = "server.jar";

export interface CreatedInstance {
  view: InstanceView;
  error: string | null;
  installStarted: boolean;
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function installCustomJar(instanceId: number, jar: File | null): Promise<InstanceView> {
  if (!jar) throw new Error("No server jar was selected.");
  await uploadFile({
    instanceId,
    path: CUSTOM_JAR_PATH,
    file: jar,
    onProgress: () => undefined,
    signal: new AbortController().signal,
  });
  return api.post<InstanceView>(`/instances/${instanceId}/install`);
}

export function useCreateInstance() {
  const queryClient = useQueryClient();
  return useMutation<CreatedInstance, ApiError, CreateDraft>({
    mutationFn: async (draft) => {
      let view = await api.post<InstanceView>("/instances", createBody(draft));
      putInstance(queryClient, view);
      const errors: string[] = [];
      try {
        view = await api.patch<InstanceView>(`/instances/${view.id}`, settingsPatch(draft));
        putInstance(queryClient, view);
      } catch (error) {
        errors.push(`its settings were not saved: ${messageOf(error)}`);
      }
      let installStarted = true;
      if (isCustom(draft)) {
        try {
          view = await installCustomJar(view.id, draft.customJar);
          putInstance(queryClient, view);
        } catch (error) {
          installStarted = false;
          errors.push(`its server.jar could not be installed: ${messageOf(error)}`);
        }
      }
      const error = errors.length ? `The server was created, but ${errors.join("; ")}.` : null;
      return { view, error, installStarted };
    },
  });
}
