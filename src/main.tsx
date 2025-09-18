import React from "react";
import {BrowserRouter, Route, Routes, useLocation, useNavigate} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";
import {AnimatePresence} from "framer-motion";

import "./assets/css/index.css";
import {ThemeProvider} from "./providers/ThemeProvider.tsx";
import {HeroUIProvider, ToastProvider} from "@heroui/react";
import {AuthenticationProvider} from "./providers/AuthenticationProvider.tsx";
import {HostInfoProvider} from "./providers/HostInfoProvider.tsx";
import Footer from "./components/Footer.tsx";
import {MinecraftVersionsProvider} from "./providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";
import {ServerProvider} from "./providers/ServerProvider.tsx";
import {MessageProvider} from "./providers/MessageProvider.tsx";
import {JavaVersionProvider} from "./providers/JavaVersionProvider.tsx";
import {WindowProvider} from "./providers/WindowProvider.tsx";
import Login from "./pages/Login.tsx";
import Dashboard from "./pages/Dashboard.tsx";
import ServerPage from "./pages/ServerPage.tsx";
import DiscoverPage from "./pages/DiscoverPage.tsx";
import {ContentPage} from "./pages/ContentPage.tsx";
import ErrorPage from "./pages/ErrorPage.tsx";
import Navigation from "./components/navigation/Navigation.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <WindowProvider>
                <ThemeProvider>
                    <MessageProvider>
                        <HostInfoProvider>
                            <AuthenticationProvider>
                                <MinecraftVersionsProvider>
                                    <ServerProvider>
                                        <JavaVersionProvider>
                                            <MainContentRenderer/>
                                        </JavaVersionProvider>
                                    </ServerProvider>
                                </MinecraftVersionsProvider>
                            </AuthenticationProvider>
                        </HostInfoProvider>
                    </MessageProvider>
                </ThemeProvider>
            </WindowProvider>
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
                            <Route path="/app/servers/:id" element={<ServerPage/>}/>
                            <Route path="/app/discover/:type" element={<DiscoverPage/>}/>
                            <Route path="/app/discover/:type/:platform" element={<DiscoverPage/>}/>
                            <Route path="/app/discover/:type/:platform/:modId" element={<ContentPage/>}/>
                            <Route path="*" element={<ErrorPage/>}/>
                        </Route>
                    </Routes>
                    <Footer/>
                </main>
            </AnimatePresence>
        </HeroUIProvider>
    );
}