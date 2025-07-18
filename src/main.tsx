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
import {ServerInfoProvider} from "./assets/providers/ServerInfoProvider.tsx";
import Footer from "./assets/components/Footer.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <ServerInfoProvider>
                    <AuthenticationProvider>
                        <MainContentRenderer/>
                    </AuthenticationProvider>
                </ServerInfoProvider>
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
                <main className={"w-full h-screen flex flex-col"}>
                    <Navigation/>
                    <Routes location={location} key={location.pathname}>
                        <Route>
                            <Route path="/" element={<Login/>}/>
                            <Route path="/app" element={<Dashboard/>}/>
                        </Route>
                    </Routes>
                    <Footer/>
                </main>
            </AnimatePresence>
        </HeroUIProvider>
    );
}