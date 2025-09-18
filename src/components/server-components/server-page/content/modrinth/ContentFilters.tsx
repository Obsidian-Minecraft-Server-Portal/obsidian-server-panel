import {GameVersionSelector} from "../GameVersionSelector.tsx";
import {LoaderSelector} from "../LoaderSelector.tsx";
import {CategorySelectorSelector} from "./CategorySelector.tsx";

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
    const {selectedGameVersions, selectedLoaders, selectedCategories, onGameVersionChange, onLoaderChange, onCategoryChange} = props;
    return (
        <>
            <GameVersionSelector
                selectedKeys={selectedGameVersions}
                onSelectionChange={keys => onGameVersionChange([...keys] as string[])}
            />
            <LoaderSelector
                selectedKeys={selectedLoaders}
                onSelectionChange={keys => onLoaderChange([...keys] as string[])}
            />
            <CategorySelectorSelector
                selectedKeys={selectedCategories}
                onSelectionChange={keys => onCategoryChange([...keys] as string[])}
            />
        </>
    );
}