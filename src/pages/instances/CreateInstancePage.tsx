import { useState } from "react";
import { useNavigate } from "react-router";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { InstallProgress } from "../../components/instance/wizard/InstallProgress";
import { INITIAL_DRAFT, STEP_LABELS, canAdvance, type WizardDraft } from "../../components/instance/wizard/draft";
import { StepDetails } from "../../components/instance/wizard/StepDetails";
import { StepIndicator } from "../../components/instance/wizard/StepIndicator";
import { StepJava } from "../../components/instance/wizard/StepJava";
import { StepServerType } from "../../components/instance/wizard/StepServerType";
import { StepVersion } from "../../components/instance/wizard/StepVersion";
import { useCreateWizard } from "../../components/instance/wizard/useCreateWizard";

const STEPS = [StepServerType, StepVersion, StepJava, StepDetails] as const;
const LAST_STEP = STEPS.length - 1;

function withDependentResets(draft: WizardDraft, next: Partial<WizardDraft>): WizardDraft {
  if ("loader" in next && next.loader !== draft.loader) {
    return { ...draft, mcVersion: null, loaderVersion: null, customJar: null, ...next };
  }
  if ("mcVersion" in next && next.mcVersion !== draft.mcVersion) {
    return { ...draft, loaderVersion: null, ...next };
  }
  return { ...draft, ...next };
}

export function CreateInstancePage() {
  const navigate = useNavigate();
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState(INITIAL_DRAFT);
  const create = useCreateWizard();
  const created = create.data;
  const Current = STEPS[step];
  const ready = canAdvance(step, draft);
  const isLast = step === LAST_STEP;

  const patch = (next: Partial<WizardDraft>) => setDraft((current) => withDependentResets(current, next));

  return (
    <>
      <PageHeader
        title="New instance"
        subtitle="Server type, Minecraft version, loader and Java."
        actions={<GhostButton onPress={() => navigate("/instances")}>Cancel</GhostButton>}
      />
      <div className="space-y-6 px-8 pb-8">
        {created ? (
          <>
            {created.error && <p className="text-sm text-amber">{created.error}</p>}
            {created.installStarted ? (
              <InstallProgress view={created.view} />
            ) : (
              <GhostButton onPress={() => navigate(`/instances/${created.view.id}`)}>Open instance</GhostButton>
            )}
          </>
        ) : (
          <>
            <StepIndicator current={step} maxReachable={step} onSelect={setStep} />
            <Current draft={draft} patch={patch} />
            <div className="flex items-center justify-between gap-3">
              <div className="min-h-5 text-sm text-rose">{create.error?.message}</div>
              <div className="flex gap-2">
                <GhostButton isDisabled={step === 0 || create.isPending} onPress={() => setStep(step - 1)}>
                  Back
                </GhostButton>
                {isLast ? (
                  <PrimaryButton isPending={create.isPending} isDisabled={!ready} onPress={() => create.mutate(draft)}>
                    Create server
                  </PrimaryButton>
                ) : (
                  <PrimaryButton isDisabled={!ready} onPress={() => setStep(step + 1)}>
                    Next · {STEP_LABELS[step + 1]}
                  </PrimaryButton>
                )}
              </div>
            </div>
          </>
        )}
      </div>
    </>
  );
}
