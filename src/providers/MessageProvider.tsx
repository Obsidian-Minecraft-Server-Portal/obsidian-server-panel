import {createContext, ReactNode, useContext, useState} from "react";
import MessageModal, {MessageOptions, MessageResponseType} from "../components/MessageModal.tsx";

// Create separate option types for type safety
type YesNoOptions = MessageOptions & { responseType: MessageResponseType.YesNo };
type OkayCancelOptions = MessageOptions & { responseType: MessageResponseType.OkayCancel };
type CloseOptions = MessageOptions & { responseType: MessageResponseType.Close };

interface MessageContextType
{
    open(options: YesNoOptions): Promise<boolean>;

    open(options: OkayCancelOptions): Promise<boolean>;

    open(options: CloseOptions): Promise<void>;

    open(options: MessageOptions): Promise<boolean | null>;
}

const MessageContext = createContext<MessageContextType | undefined>(undefined);

export function MessageProvider({children}: { children: ReactNode })
{
    const [messageOptions, setMessageOptions] = useState<MessageOptions | null>(null);
    const [resolvePromise, setResolvePromise] = useState<((value: boolean | null | void) => void) | null>(null);

    const open = (options: MessageOptions): Promise<boolean | null | void> =>
    {
        return new Promise((resolve) =>
        {
            setMessageOptions(options);
            setResolvePromise(() => resolve);
        });
    };

    const handleClose = (response: boolean | null) =>
    {
        setMessageOptions(null);
        if (resolvePromise)
        {
            if (messageOptions?.responseType === MessageResponseType.Close)
            {
                resolvePromise(undefined); // void for Close type
            } else
            {
                resolvePromise(response === null ? false : response); // guarantee boolean for YesNo/OkayCancel
            }
            setResolvePromise(null);
        }
    };

    return (
        <MessageContext.Provider value={{open: open as MessageContextType["open"]}}>
            <MessageModal
                isOpen={messageOptions != null}
                title={messageOptions?.title ?? ""}
                body={messageOptions?.body ?? ""}
                responseType={messageOptions?.responseType ?? MessageResponseType.Close}
                severity={messageOptions?.severity}
                onClose={handleClose}
            />
            {children}
        </MessageContext.Provider>
    );
}

export function useMessage(): MessageContextType
{
    const context = useContext(MessageContext);
    if (!context)
    {
        throw new Error("useMessage must be used within a MessageProvider");
    }
    return context;
}