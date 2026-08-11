export interface AjaxOptions
{
    url?: string;
    method?: string;
    type?: string;
    data?: BodyInit | null;
    contentType?: string | false;
    dataType?: string;
    timeout?: number;
    headers?: Record<string, string>;
}

export interface HttpError extends Error
{
    status: number;
    statusText: string;
    responseText?: string;
    responseJSON?: {message?: string; error?: string; [key: string]: unknown};
}

async function request(url: string, options: AjaxOptions = {})
{
    const method = (options.method ?? options.type ?? "GET").toUpperCase();
    const headers: Record<string, string> = {...options.headers};
    if (options.contentType) headers["Content-Type"] = options.contentType;
    const response = await fetch(url, {
        method,
        headers,
        body: options.data ?? undefined,
        credentials: "same-origin"
    });
    const text = await response.text();
    let json: unknown;
    try
    {
        json = text ? JSON.parse(text) : undefined;
    } catch
    {
        json = undefined;
    }
    if (!response.ok)
    {
        const err = new Error(`${response.status} ${response.statusText}`) as HttpError;
        err.status = response.status;
        err.statusText = response.statusText;
        err.responseText = text;
        err.responseJSON = json as HttpError["responseJSON"];
        if (err.responseJSON?.message) err.message = String(err.responseJSON.message);
        throw err;
    }
    return json ?? text;
}

const $ = {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    get: (url: string): Promise<any> => request(url),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    post: (url: string, data?: BodyInit): Promise<any> => request(url, {method: "POST", data}),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ajax: (urlOrOptions: string | AjaxOptions, options?: AjaxOptions): Promise<any> =>
    {
        if (typeof urlOrOptions === "string") return request(urlOrOptions, options);
        return request(urlOrOptions.url ?? "", urlOrOptions);
    }
};

export default $;
