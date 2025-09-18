import {useNavigate, useParams} from "react-router-dom";
import {useEffect} from "react";

export default function DiscoverPage()
{
    const {type, platform} = useParams();
    const navigate = useNavigate();
    useEffect(() =>
    {
        if(!platform)navigate(`/app/discover/${type}/modrinth`)
    }, [type, platform]);
    return (
        <>
        </>
    );
}