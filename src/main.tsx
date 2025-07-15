import React from "react";
import {BrowserRouter, Route, Routes, useNavigate} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";

import "./assets/css/index.css";
import {ThemeProvider} from "./assets/providers/ThemeProvider.tsx";
import {HeroUIProvider} from "@heroui/react";
import Login from "./assets/pages/Login.tsx";
import Dashboard from "./assets/pages/Dashboard.tsx";


ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <MainContentRenderer/>
            </ThemeProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
    return (
        <HeroUIProvider navigate={navigate}>
            <Routes>
                <Route>
                    <Route path="/" element={<Login/>}/>
                    <Route path="/app" element={<Dashboard/>}/>
                </Route>
            </Routes>
        </HeroUIProvider>
    );
}
