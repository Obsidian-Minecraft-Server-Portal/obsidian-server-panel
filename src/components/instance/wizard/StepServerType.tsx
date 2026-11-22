import { LoaderPicker } from "../../runtime/LoaderPicker";
import { Panel } from "../../ui/Panel";
import type { WizardDraft } from "./draft";

interface StepProps {
  draft: WizardDraft;
  patch: (next: Partial<WizardDraft>) => void;
}

export function StepServerType({ draft, patch }: StepProps) {
  return (
    <Panel title="Choose a server type">
      <LoaderPicker value={draft.loader} onChange={(loader) => patch({ loader })} />
    </Panel>
  );
}
