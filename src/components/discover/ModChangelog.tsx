import { Button, Card, CardBody, Chip } from "@heroui/react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { ChangelogEntry } from "../../types/ModTypes";

interface ModChangelogProps {
    changelog: ChangelogEntry[];
    changelogPage: number;
    onLoadMore: () => void;
}

const CHANGELOG_PER_PAGE = 5;

export function ModChangelog({ changelog, changelogPage, onLoadMore }: ModChangelogProps) {
    const getVersionTypeIcon = (type: string) => {
        switch (type) {
            case "release":
                return "R";
            case "beta":
                return "B";
            case "alpha":
                return "A";
            default:
                return "?";
        }
    };

    const getVersionTypeColor = (type: string) => {
        switch (type) {
            case "release":
                return "success";
            case "beta":
                return "warning";
            case "alpha":
                return "danger";
            default:
                return "default";
        }
    };

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString();
    };

    const filteredChangelog = changelog.slice(0, changelogPage * CHANGELOG_PER_PAGE);

    return (
        <Card radius="none">
            <CardBody className="p-6 min-h-[calc(100dvh_-_440px)]">
                {changelog.length > 0 ? (
                    <div className="space-y-6">
                        {filteredChangelog.map((entry, index) => (
                            <div key={index} className="border-l-4 border-primary pl-4">
                                <div className="flex items-center gap-3 mb-2">
                                    <Chip
                                        size="sm"
                                        color={getVersionTypeColor(entry.version_type) as any}
                                        variant="flat"
                                    >
                                        {getVersionTypeIcon(entry.version_type)} {entry.version}
                                    </Chip>
                                    <span className="text-default-500 text-sm">
                                        {formatDate(entry.date)}
                                    </span>
                                </div>
                                <div className="prose prose-sm max-w-none">
                                    <ReactMarkdown
                                        remarkPlugins={[remarkGfm]}
                                        rehypePlugins={[rehypeRaw, rehypeSanitize]}
                                    >
                                        {entry.changes}
                                    </ReactMarkdown>
                                </div>
                            </div>
                        ))}
                        {changelog.length > changelogPage * CHANGELOG_PER_PAGE && (
                            <div className="flex justify-center">
                                <Button
                                    variant="ghost"
                                    color="primary"
                                    onPress={onLoadMore}
                                    radius="none"
                                >
                                    Load more
                                </Button>
                            </div>
                        )}
                    </div>
                ) : (
                    <p className="text-default-500">No changelog available</p>
                )}
            </CardBody>
        </Card>
    );
}
