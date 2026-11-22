import { useState, type FormEvent } from "react";
import { useUpdateInstance, type InstanceView, type UpdateInstanceInput } from "../../../api/instances";
import { JavaSelect } from "../../runtime/JavaSelect";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { FormField, SwitchField } from "../FormField";
import { isMemoryRange, isPortValid } from "../wizard/draft";

interface Form {
  name: string;
  port: string;
  jvmArgs: string;
  memoryMinMb: string;
  memoryMaxMb: string;
  javaRuntimeId: number | null;
  autoStart: boolean;
}

function initialForm(view: InstanceView): Form {
  return {
    name: view.name,
    port: String(view.port),
    jvmArgs: view.jvm_args,
    memoryMinMb: String(view.memory_min_mb),
    memoryMaxMb: String(view.memory_max_mb),
    javaRuntimeId: view.java_runtime_id,
    autoStart: view.auto_start,
  };
}

function changes(view: InstanceView, form: Form): UpdateInstanceInput {
  const patch: UpdateInstanceInput = {};
  const name = form.name.trim();
  const port = Number(form.port);
  const memoryMin = Number(form.memoryMinMb);
  const memoryMax = Number(form.memoryMaxMb);
  if (name !== view.name) patch.name = name;
  if (port !== view.port) patch.port = port;
  if (form.jvmArgs !== view.jvm_args) patch.jvm_args = form.jvmArgs;
  if (memoryMin !== view.memory_min_mb) patch.memory_min_mb = memoryMin;
  if (memoryMax !== view.memory_max_mb) patch.memory_max_mb = memoryMax;
  if (form.javaRuntimeId !== view.java_runtime_id) patch.java_runtime_id = form.javaRuntimeId;
  if (form.autoStart !== view.auto_start) patch.auto_start = form.autoStart;
  return patch;
}

function memoryValid(form: Form): boolean {
  return isMemoryRange(Number(form.memoryMinMb), Number(form.memoryMaxMb));
}

export function generalSettingsKey(view: InstanceView): string {
  return JSON.stringify(initialForm(view));
}

export function GeneralSettings({ view }: { view: InstanceView }) {
  const [form, setForm] = useState(() => initialForm(view));
  const update = useUpdateInstance(view.id);
  const patch = changes(view, form);
  const dirty = Object.keys(patch).length > 0;
  const memoryOk = memoryValid(form);
  const portOk = form.port !== "" && isPortValid(form.port);
  const valid = form.name.trim().length > 0 && portOk && memoryOk;
  const set = <K extends keyof Form>(key: K, value: Form[K]) => setForm((current) => ({ ...current, [key]: value }));

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (dirty && valid) update.mutate(patch);
  };

  return (
    <Panel title="General">
      <form onSubmit={submit} className="space-y-5">
        <FormField label="Name" value={form.name} onChange={(value) => set("name", value)} />
        <div className="grid gap-5 md:grid-cols-2">
          <FormField
            label="Port"
            type="number"
            value={form.port}
            onChange={(value) => set("port", value)}
            hint={portOk ? "Applies the next time the server starts." : "Enter a port between 1 and 65535."}
          />
          <JavaSelect value={form.javaRuntimeId} onChange={(id) => id !== null && set("javaRuntimeId", id)} isDisabled={update.isPending} />
          <FormField
            label="Memory min (MB)"
            type="number"
            value={form.memoryMinMb}
            onChange={(value) => set("memoryMinMb", value)}
          />
          <FormField
            label="Memory max (MB)"
            type="number"
            value={form.memoryMaxMb}
            onChange={(value) => set("memoryMaxMb", value)}
            hint={memoryOk ? undefined : "Whole numbers; min must be positive and not above max."}
          />
        </div>
        <FormField
          label="JVM arguments"
          multiline
          rows={3}
          value={form.jvmArgs}
          onChange={(value) => set("jvmArgs", value)}
          hint="Extra flags passed to Java before the server jar."
        />
        <SwitchField
          label="Start automatically"
          hint="Start this server when the panel boots."
          isSelected={form.autoStart}
          onChange={(value) => set("autoStart", value)}
        />
        <div className="flex items-center justify-between gap-3">
          {update.error ? <p className="text-sm text-rose">{update.error.message}</p> : <span />}
          <PrimaryButton type="submit" isPending={update.isPending} isDisabled={!dirty || !valid}>
            Save settings
          </PrimaryButton>
        </div>
      </form>
    </Panel>
  );
}
