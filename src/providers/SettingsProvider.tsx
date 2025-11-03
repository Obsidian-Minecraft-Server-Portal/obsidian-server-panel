import $ from "jquery";
import {createContext, ReactNode, useContext, useEffect, useState} from "react";
import {Settings, ValidationResult, SettingsUpdateResponse} from "../types/SettingsTypes.ts";

interface SettingsContextType {
    settings: Settings | null;
    loading: boolean;
    error: string | null;
    refreshSettings: () => Promise<void>;
    updateSettings: (newSettings: Settings) => Promise<SettingsUpdateResponse>;
    validateSettings: (settings: Settings) => Promise<ValidationResult>;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export function SettingsProvider({children}: { children: ReactNode }) {
    const [settings, setSettings] = useState<Settings | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    const refreshSettings = async () => {
        try {
            setLoading(true);
            setError(null);
            const response: Settings = await $.get("/api/settings");
            setSettings(response);
        } catch (err: any) {
            console.error("Failed to load settings:", err);
            const errorMessage = err.responseJSON?.message || "Failed to load settings";
            setError(errorMessage);
        } finally {
            setLoading(false);
        }
    };

    const updateSettings = async (newSettings: Settings) => {
        try {
            setLoading(true);
            setError(null);
            const response = await $.ajax("/api/settings", {
                method: "PUT",
                data: JSON.stringify(newSettings),
                contentType: "application/json"
            });
            setSettings(response.settings);
            return response; // Return the full response including migration_info
        } catch (err: any) {
            console.error("Failed to update settings:", err);
            const errorMessage = err.responseJSON?.message || "Failed to update settings";
            setError(errorMessage);
            throw new Error(errorMessage);
        } finally {
            setLoading(false);
        }
    };

    const validateSettings = async (settingsToValidate: Settings): Promise<ValidationResult> => {
        try {
            const response: ValidationResult = await $.ajax("/api/settings/validate", {
                method: "POST",
                data: JSON.stringify(settingsToValidate),
                contentType: "application/json"
            });
            return response;
        } catch (err: any) {
            console.error("Failed to validate settings:", err);
            return {
                valid: false,
                message: "Validation failed",
                error: err.responseJSON?.message || "Unknown error"
            };
        }
    };

    useEffect(() => {
        refreshSettings().catch(console.error);
    }, []);

    return (
        <SettingsContext.Provider value={{
            settings,
            loading,
            error,
            refreshSettings,
            updateSettings,
            validateSettings
        }}>
            {children}
        </SettingsContext.Provider>
    );
}

export function useSettings(): SettingsContextType {
    const context = useContext(SettingsContext);
    if (!context) {
        throw new Error("useSettings must be used within a SettingsProvider");
    }
    return context;
}
