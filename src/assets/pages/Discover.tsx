import {useParams, useSearchParams} from "react-router-dom";

export function Discover()
{
    const {type, platform, modId} = useParams();
    const [searchParams] = useSearchParams();
    let serverId = searchParams.get("sid");
    return (
        <>
        </>
    );
}