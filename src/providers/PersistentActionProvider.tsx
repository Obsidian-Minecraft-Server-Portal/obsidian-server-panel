import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";

export interface ActionData
{
    id: number;
    user_id: number;
    tracker_id: string;
    action_type: "archive" | "extract" | "move" | "copy" | "upload" | "backup_create" | "mod_download";
    status: "in_progress" | "completed" | "failed";
    progress: number;
    details?: string;
    created_at: string;
    updated_at: string;
    completed_at?: string;
}

interface PersistentActionContextType
{
    actions: ActionData[];
    activeActions: ActionData[];
    loading: boolean;
    error: string | null;
    refreshActions: () => Promise<void>;
    clearCompletedActions: () => Promise<void>;
    deleteAction: (trackerId: string) => Promise<void>;
}

const PersistentActionContext = createContext<PersistentActionContextType | undefined>(undefined);

export function PersistentActionProvider({children}: { children: ReactNode })
{
    const [actions, setActions] = useState<ActionData[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const activeActions = actions.filter(action => action.status === "in_progress");

    const fetchActions = useCallback(async () =>
    {
        setLoading(true);
        setError(null);

        try
        {
            const response = await fetch("/api/actions", {
                headers: {
                    "Content-Type": "application/json"
                },
                credentials: "include"
            });

            if (!response.ok)
            {
                throw new Error(`Failed to fetch actions: ${response.statusText}`);
            }

            const data = await response.json();
            setActions(data);
        } catch (err)
        {
            setError(err instanceof Error ? err.message : "Unknown error occurred");
            console.error("Failed to fetch actions:", err);
        } finally
        {
            setLoading(false);
        }
    }, []);

    const refreshActions = useCallback(async () =>
    {
        await fetchActions();
    }, [fetchActions]);

    const clearCompletedActions = useCallback(async () =>
    {
        try
        {
            const response = await fetch("/api/actions/completed", {
                method: "DELETE",
                headers: {
                    "Content-Type": "application/json"
                },
                credentials: "include"
            });

            if (!response.ok)
            {
                throw new Error(`Failed to clear completed actions: ${response.statusText}`);
            }

            await fetchActions(); // Refresh after clearing
        } catch (err)
        {
            setError(err instanceof Error ? err.message : "Failed to clear completed actions");
            console.error("Failed to clear completed actions:", err);
        }
    }, [fetchActions]);

    const deleteAction = useCallback(async (trackerId: string) =>
    {
        try
        {
            const response = await fetch(`/api/actions/${trackerId}`, {
                method: "DELETE",
                headers: {
                    "Content-Type": "application/json"
                },
                credentials: "include"
            });

            if (!response.ok)
            {
                throw new Error(`Failed to delete action: ${response.statusText}`);
            }

            await fetchActions(); // Refresh after deleting
        } catch (err)
        {
            setError(err instanceof Error ? err.message : "Failed to delete action");
            console.error("Failed to delete action:", err);
        }
    }, [fetchActions]);

    // Fetch actions on mount and listen for WebSocket updates
    useEffect(() =>
    {
        // Fetch actions once on mount
        fetchActions();

        // Listen for action updates via WebSocket
        const handleActionUpdate = () => {
            fetchActions();
        };

        const handleActionComplete = () => {
            fetchActions();
        };

        window.addEventListener('action-update', handleActionUpdate);
        window.addEventListener('action-complete', handleActionComplete);

        return () => {
            window.removeEventListener('action-update', handleActionUpdate);
            window.removeEventListener('action-complete', handleActionComplete);
        };
    }, [fetchActions]);

    const value = {
        actions,
        activeActions,
        loading,
        error,
        refreshActions,
        clearCompletedActions,
        deleteAction
    };

    return (
        <PersistentActionContext.Provider value={value}>
            {children}
        </PersistentActionContext.Provider>
    );
}

export function usePersistentAction(): PersistentActionContextType
{
    const context = useContext(PersistentActionContext);
    if (!context)
    {
        throw new Error("usePersistentAction must be used within a PersistentActionProvider");
    }
    return context;
}