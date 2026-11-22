import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { useServerPackDraft, type ServerPackReceipt } from "../../api/serverpack";
import { DoneStep } from "../../components/serverpack/DoneStep";
import { ExportStep } from "../../components/serverpack/ExportStep";
import { ReviewStep, type Decisions } from "../../components/serverpack/ReviewStep";
import { LOADER_META } from "../../components/runtime/loaderMeta";
import { GhostButton } from "../../components/ui/GhostButton";

const STEPS = ["Review", "Export", "Done"];

export function ServerPackPage() {
  const { draftId = "" } = useParams();
  const navigate = useNavigate();
  const draft = useServerPackDraft(draftId);
  const [step, setStep] = useState(0);
  const [decisions, setDecisions] = useState<Decisions>({ mods: {}, folders: {} });
  const [receipt, setReceipt] = useState<ServerPackReceipt | null>(null);

  if (draft.isError) {
    return (
      <div className="space-y-4 px-8 py-6">
        <p className="text-sm text-rose">{draft.error.message}</p>
        <p className="text-sm text-ink-3">This server pack draft may have expired. Start again from Discover.</p>
        <Link to="/discover" className="text-sm font-medium text-accent hover:underline">
          Back to Discover
        </Link>
      </div>
    );
  }

  if (!draft.data) return <p className="px-8 py-6 text-sm text-ink-3">Loading the modpack…</p>;

  const data = draft.data;
  return (
    <div className="space-y-6 px-8 py-6">
      <header className="flex flex-wrap items-center gap-4">
        <GhostButton isIconOnly aria-label="Back to Discover" onPress={() => navigate("/discover")} className="size-10">
          <Icon icon="lucide:chevron-left" width={18} height={18} />
        </GhostButton>
        <div className="min-w-0 flex-1 space-y-1">
          <h1 className="truncate text-2xl font-semibold tracking-tight text-ink-0">Build server pack</h1>
          <p className="truncate font-mono text-[11px] text-ink-3">
            {data.pack_name} · {LOADER_META[data.loader].label} {data.mc_version}
          </p>
        </div>
        <ol aria-label="Steps" className="flex items-center gap-4 font-mono text-[11px] tracking-wider text-ink-3 uppercase">
          {STEPS.map((label, index) => (
            <li
              key={label}
              aria-current={index === step ? "step" : undefined}
              className={index === step ? "text-accent" : index < step ? "text-ink-2" : undefined}
            >
              {index + 1}. {label}
            </li>
          ))}
        </ol>
      </header>

      {step === 0 && (
        <ReviewStep
          draft={data}
          decisions={decisions}
          onDecisionsChange={setDecisions}
          onBack={() => navigate("/discover")}
          onContinue={() => setStep(1)}
        />
      )}
      {step === 1 && (
        <ExportStep
          draft={data}
          decisions={decisions}
          onBack={() => setStep(0)}
          onDone={(next) => {
            setReceipt(next);
            setStep(2);
          }}
        />
      )}
      {step === 2 && receipt && <DoneStep receipt={receipt} />}
    </div>
  );
}
