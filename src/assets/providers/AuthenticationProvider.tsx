import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import $ from "jquery";
import {useLocation, useNavigate} from "react-router-dom";

export type UserData = {
    id: string | null,
    username: string,
    permissions: string,
    join_date: Date,
    last_online: Date,
}

type LoginResponse = {
    user?: UserData,
    message: string,
    stacktrace?: any,
}

interface AuthenticationContextType
{
    isAuthenticated: boolean | undefined;
    user: UserData | null;
    login: (username: string, password: string, rememberMe: boolean, delay?: number) => Promise<void>;
    loginWithToken: () => Promise<void>;
    logout: () => void;
    register: (username: string, password: string) => Promise<void>;
    isLoggingIn: boolean;
}

const AuthenticationContext = createContext<AuthenticationContextType | undefined>(undefined);

export function AuthenticationProvider({children}: { children: ReactNode })
{
    const [user, setUser] = useState<UserData | null>(null);
    const [isAuthenticated, setIsAuthenticated] = useState<boolean | undefined>(undefined);
    const [isLoggingIn, setIsLoggingIn] = useState<boolean>(false);
    const navigate = useNavigate();
    const {pathname} = useLocation();

    const login = useCallback(async (username: string, password: string, rememberMe: boolean, delay?: number) =>
    {
        setIsLoggingIn(true);
        try
        {
            const response: LoginResponse = await $.ajax("/api/auth/", {
                method: "POST",
                data: JSON.stringify({username, password, remember: rememberMe}),
                contentType: "application/json",
                dataType: "json"
            });
            console.log("Login Response: ", response);
            if (response.user)
            {
                setUser(response.user);
                if (delay)
                {
                    console.log(`Delaying login for ${delay}ms`);
                    await new Promise(resolve => setTimeout(resolve, delay));
                }
                setIsAuthenticated(true);
                setTimeout(() => setIsLoggingIn(false), 1000);
            } else
            {
                throw new Error(response.message || "Login failed");
            }
        } catch (err: any | Error)
        {
            const errorMessage = err.responseJSON?.message || err.message;
            console.error(`Failed to login`, errorMessage);
            setIsAuthenticated(false);
            setTimeout(() => setIsLoggingIn(false), 5000);
            throw new Error(errorMessage || "Failed to login");
        }

    }, [setUser, setIsAuthenticated, setIsLoggingIn]);

    const loginWithToken = useCallback(async () =>
    {
        setIsLoggingIn(true);
        try
        {
            const response: LoginResponse = await $.get("/api/auth/");
            console.log("Token Login Response: ", response);
            if (response.user)
            {
                setUser(response.user);
                setIsAuthenticated(true);
            } else
            {
                throw new Error(response.message || "Token login failed");
            }
        } catch (err: any | Error)
        {
            const errorMessage = err.responseJSON?.message || err.message;
            console.error(`Failed to login with token`, errorMessage);
            setIsAuthenticated(false);
            throw new Error(errorMessage || "Failed to login with token");
        } finally
        {
            setIsLoggingIn(false);
        }

    }, [setUser, setIsAuthenticated, setIsLoggingIn]);

    const logout = useCallback(async () =>
    {
        setUser(null);
        setIsAuthenticated(false);
        await $.get("/api/auth/logout/");
    }, [setUser, setIsAuthenticated]);


    const register = useCallback(async (username: string, password: string) =>
    {
        setIsLoggingIn(true);
        try
        {
            const response: LoginResponse = await $.ajax("/api/auth/", {
                method: "PUT",
                data: JSON.stringify({username, password}),
                contentType: "application/json",
                dataType: "json"
            });
            console.log("Register Response: ", response);
        } catch (err: any | Error)
        {
            const errorMessage = err.responseJSON?.message || err.message;
            console.error(`Failed to register`, errorMessage);
            throw new Error(errorMessage || "Failed to register");
        } finally
        {
            setIsLoggingIn(false);
        }
    }, [setUser, setIsAuthenticated, setIsLoggingIn]);


    useEffect(() =>
    {
        loginWithToken();
    }, []);


    useEffect(() =>
    {
        if (isAuthenticated === undefined) return; // Wait for the initial authentication check
        if (isAuthenticated)
        {
            if (!pathname.startsWith("/app"))
            {
                console.log("User is authenticated, redirecting to dashboard...");
                navigate("/app");
            }
        } else
        {
            if (pathname.startsWith("/app"))
            {
                console.warn("User is not authenticated, redirecting to login...");
                navigate("/");
            }
        }
    }, [isAuthenticated, pathname]);


    return (
        <AuthenticationContext.Provider value={{user, isAuthenticated, login, logout, loginWithToken, isLoggingIn, register}}>
            {children}
        </AuthenticationContext.Provider>
    );
}

export function useAuthentication(): AuthenticationContextType
{
    const context = useContext(AuthenticationContext);
    if (!context)
    {
        throw new Error("useAuthentication must be used within a AuthenticationProvider");
    }
    return context;
}