import { useState } from "react";
import { useNavigate } from "react-router";
import { GhostButton } from "../../components/ui/GhostButton";
import { PageHeader } from "../../components/ui/PageHeader";
import { InstallProgress } from "../../components/instance/create/InstallProgress";
import {
  DetailsSection,
  OptionsSection,
  ServerTypeSection,
  VersionSection,
} from "../../components/instance/create/CreateSections";
import { CreateSummary } from "../../components/instance/create/CreateSummary";
import { INITIAL_DRAFT, withDependentResets, type CreateDraft } from "../../components/instance/create/draft";
import { useCreateInstance } from "../../components/instance/create/useCreateInstance";

export function CreateInstancePage() {
  const navigate = useNavigate();
  const [draft, setDraft] = useState(INITIAL_DRAFT);
  const create = useCreateInstance();
  const created = create.data;

  const patch = (next: Partial<CreateDraft>) => setDraft((current) => withDependentResets(current, next));

  return (
    <>
      <PageHeader
        title="New server"
        subtitle="Pick the software, version and resources. Everything can be changed later."
        actions={<GhostButton onPress={() => navigate("/")}>Cancel</GhostButton>}
      />
      <div className="px-8 pb-10">
        {created ? (
          <div className="mx-auto max-w-2xl space-y-4">
            {created.error && <p className="text-sm text-amber">{created.error}</p>}
            {created.installStarted ? (
              <InstallProgress view={created.view} />
            ) : (
              <GhostButton onPress={() => navigate(`/instances/${created.view.id}`)}>Open instance</GhostButton>
            )}
          </div>
        ) : (
          <div className="mx-auto grid max-w-[1400px] items-start gap-6 xl:grid-cols-[minmax(0,1fr)_22rem]">
            <div className="min-w-0 space-y-6">
              <ServerTypeSection draft={draft} patch={patch} />
              <VersionSection draft={draft} patch={patch} />
              <DetailsSection draft={draft} patch={patch} />
              <OptionsSection draft={draft} patch={patch} />
            </div>
            <CreateSummary
              draft={draft}
              onEula={(acceptEula) => patch({ acceptEula })}
              onCreate={() => create.mutate(draft)}
              isPending={create.isPending}
              error={create.error?.message}
            />
          </div>
        )}
      </div>
    </>
  );
}
