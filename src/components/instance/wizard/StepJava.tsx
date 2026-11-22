import { Link } from "react-router";
import { JavaSelect } from "../../runtime/JavaSelect";
import { Panel } from "../../ui/Panel";
import type { WizardDraft } from "./draft";

interface StepProps {
  draft: WizardDraft;
  patch: (next: Partial<WizardDraft>) => void;
}

export function StepJava({ draft, patch }: StepProps) {
  return (
    <Panel title="Java runtime">
      <div className="max-w-xl space-y-4">
        <JavaSelect value={draft.javaRuntimeId} onChange={(javaRuntimeId) => patch({ javaRuntimeId })} />
        <p className="text-sm text-ink-3">
          Automatic picks the runtime the Minecraft version needs. Install managed runtimes in{" "}
          <Link to="/settings/java" className="text-accent hover:underline">
            Settings
          </Link>
          .
        </p>
      </div>
    </Panel>
  );
}
