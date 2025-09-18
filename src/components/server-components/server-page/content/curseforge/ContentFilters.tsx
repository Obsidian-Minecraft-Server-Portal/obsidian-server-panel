import {GameVersionSelector} from "../GameVersionSelector.tsx";
import {LoaderSelector} from "../LoaderSelector.tsx";

type ContentFiltersProps = {
    selectedGameVersions: string[];
    selectedLoaders: string[];
    selectedCategories: string[];
    onGameVersionChange: (versions: string[]) => void;
    onLoaderChange: (loaders: string[]) => void;
    onCategoryChange: (categories: string[]) => void;
}

export function ContentFilters(props: ContentFiltersProps)
{
    const {selectedGameVersions, selectedLoaders, onGameVersionChange, onLoaderChange} = props;
    return (
        <>
            <GameVersionSelector
                selectedKeys={selectedGameVersions}
                onSelectionChange={keys => onGameVersionChange([...keys] as string[])}
                selectionMode={"single"}
            />
            <LoaderSelector
                selectedKeys={selectedLoaders}
                onSelectionChange={keys => onLoaderChange([...keys] as string[])}
                selectionMode={"single"}
            />
        </>
    );
}