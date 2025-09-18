import {createContext, ReactNode, useContext, useState} from "react";
import $ from "jquery";

interface WindowContextType
{
    height: number;
    width: number;
}

const WindowContext = createContext<WindowContextType | undefined>(undefined);

export function WindowProvider({children}: { children: ReactNode })
{
    const [width, setWidth] = useState(window.innerWidth);
    const [height, setHeight] = useState(window.innerHeight);
    $(window).on("resize", () =>
    {
        setWidth(window.innerWidth);
        setHeight(window.innerHeight);
    });


    return (
        <WindowContext.Provider value={{width, height}}>
            {children}
        </WindowContext.Provider>
    );
}

export function useWindow(): WindowContextType
{
    const context = useContext(WindowContext);
    if (!context)
    {
        throw new Error("useWindow must be used within a WindowProvider");
    }
    return context;
}