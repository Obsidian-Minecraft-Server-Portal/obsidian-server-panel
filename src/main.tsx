import React from "react";
import {BrowserRouter, Route, Routes, useLocation, useNavigate} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";
import {AnimatePresence} from "framer-motion";

import "./assets/css/index.css";
import {ThemeProvider} from "./assets/providers/ThemeProvider.tsx";
import {HeroUIProvider, ToastProvider} from "@heroui/react";
import Login from "./assets/pages/Login.tsx";
import Dashboard from "./assets/pages/Dashboard.tsx";
import {AuthenticationProvider} from "./assets/providers/AuthenticationProvider.tsx";
import Navigation from "./assets/components/navigation/Navigation.tsx";
import {HostInfoProvider} from "./assets/providers/HostInfoProvider.tsx";
import Footer from "./assets/components/Footer.tsx";
import ErrorPage from "./assets/pages/ErrorPage.tsx";
import {MinecraftVersionsProvider} from "./assets/providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";
import {ServerProvider} from "./assets/providers/ServerProvider.tsx";
import {MessageProvider} from "./assets/providers/MessageProvider.tsx";
import {JavaVersionProvider} from "./assets/providers/JavaVersionProvider.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <HostInfoProvider>
                    <AuthenticationProvider>
                        <MinecraftVersionsProvider>
                            <ServerProvider>
                                <MessageProvider>
                                    <JavaVersionProvider>
                                        <MainContentRenderer/>
                                    </JavaVersionProvider>
                                </MessageProvider>
                            </ServerProvider>
                        </MinecraftVersionsProvider>
                    </AuthenticationProvider>
                </HostInfoProvider>
            </ThemeProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
    const location = useLocation();

    return (
        <HeroUIProvider navigate={navigate}>
            <ToastProvider
                placement={"bottom-right"}
                toastProps={{
                    radius: "none",
                    shouldShowTimeoutProgress: true,
                    timeout: 3000, // 3 second timeout for toasts,
                    classNames: {
                        title: "font-minecraft-header",
                        base: "font-minecraft-body"
                    }
                }}
            />
            <AnimatePresence mode="wait" initial={false}>
                <main className={"w-full flex flex-col"}>
                    <Navigation/>
                    <Routes location={location} key={location.pathname}>
                        <Route>
                            <Route path="/" element={<Login/>}/>
                            <Route path="/app" element={<Dashboard/>}/>
                            <Route path="*" element={<ErrorPage/>}/>
                        </Route>
                    </Routes>
                    <Footer/>
                </main>
            </AnimatePresence>
        </HeroUIProvider>
    );
}